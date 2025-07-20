use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use time::OffsetDateTime;

use crate::auth::db::user_data::{self, UserData};

#[derive(Serialize, Deserialize)]
pub struct JWTClaims {
  pub sub: String, // Subject (user UUID as string)
  pub exp: i64,    // Expiration timestamp (as Unix time)
  pub iat: i64,    // Issued at (as Unix time)
}

pub enum JWTError {
  EncodingError,
  DecodingError,
  InvalidToken,
  UserNotFound,
  InternalError,
}

pub fn create_jwt_token(user_uuid: &Uuid, jwt_secret: &str, expiration_days: i64) -> Result<String, JWTError> {
  let now = OffsetDateTime::now_utc().unix_timestamp();

  let claims = JWTClaims {
    sub: user_uuid.to_string(),
    exp: now + 60 * 60 * 24 * expiration_days,
    iat: now,
  };

  let mut header = Header::default();
  header.alg = Algorithm::HS256;
  
  if let Ok(token) = encode(&header, &claims, &EncodingKey::from_secret(jwt_secret.as_bytes())) {
    return Ok(token);
  }
  Err(JWTError::EncodingError)
}

pub fn get_jwt_claims(token: &str, secret: &str) -> Result<JWTClaims, JWTError> {
  let validation = Validation::new(Algorithm::HS256);
  if let Ok(token_data) = decode::<JWTClaims>(
    token,
    &DecodingKey::from_secret(secret.as_bytes()),
    &validation
  ) {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    if token_data.claims.iat > now {
      return Err(JWTError::InvalidToken);
    }
    return Ok(token_data.claims);
  }
 Err(JWTError::DecodingError)
}

pub async fn verify_jwt_token(pool: &PgPool, token: &str, secret: &str) -> Result<UserData, JWTError> {
  match get_jwt_claims(token, secret) {
    Ok(claims) => {
      if let Ok(uuid) = Uuid::parse_str(&claims.sub) {
        if let Ok(user_data) = user_data::get_user_by_uuid(pool, &uuid).await {
          if let Some(user_data) = user_data {
            return Ok(user_data);
          }
          return Err(JWTError::UserNotFound);
        }
        return Err(JWTError::InternalError);
      }
      Err(JWTError::InvalidToken)
    },
    Err(error) => Err(error),
  }
}

