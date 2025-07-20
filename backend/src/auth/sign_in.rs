use axum::{
  extract::{Json, State},
  http::StatusCode,
};
use axum_extra::{
    extract::TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::jwt::{self, JWTError};
use crate::auth::hashing::verify_password;
use crate::auth::google_claims::get_google_claims;
use crate::auth::db::user_data::{self, UserData};
use crate::auth::db::sign_in_attempts;
use crate::app_state::AppState;

const DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$+1F6v9OZsFvaYlTL8IPwtA$2lf7JtSOvRBZldOVGxWWgw+4uh/09TFFWJF6YGL+9co";

/*** Json Structs **/

#[derive(Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum SignInRequest {
  PASSWORD { email: String, password: String },
  GOOGLE { id_token: String },
}

#[derive(Serialize)]
pub struct SignInResponse {
  error_code: Option<SignInError>,
  jwt_token: Option<String>,
  user_data: Option<UserData>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SignInError {
  InvalidCredentials,
  TokenExpired,
  InternalError,
  NeedToVerifyEmail,
}

/*** Helpers ***/

async fn success_response(app_state: &AppState, user: UserData, method: &str) -> (StatusCode, Json<SignInResponse>) {
  match jwt::create_jwt_token(&user.uuid, &app_state.jwt_secret, app_state.jwt_expiration_days) {
    Ok(jwt_token) => {
      if user_data::update_last_seen(&app_state.pool, &user.uuid).await.is_err() {
        return internal_error_response();
      };
      tracing::debug!(
        event = "sign_in_success",
        method = method,
        user_uuid = %user.uuid,
      );
      (StatusCode::OK, Json(SignInResponse {
        error_code: None,
        jwt_token: Some(jwt_token),
        user_data: Some(user),
      }))
    },
    Err(error) => {
      match error {
        JWTError::InternalError => internal_error_response(),
        _ => unauthorized_response(SignInError::InvalidCredentials),
      }
    },
  }
}

fn unauthorized_response(code: SignInError) -> (StatusCode, Json<SignInResponse>) {
  tracing::warn!(
    event = "sign_in_failure",
    error_code = ?code,
  );
  (StatusCode::UNAUTHORIZED, Json(SignInResponse {
      error_code: Some(code),
      jwt_token: None,
      user_data: None,
    }),
  )
}

fn internal_error_response() -> (StatusCode, Json<SignInResponse>) {
  tracing::error!(
    event = "sign_in_failure",
    error_code = ?(SignInError::InternalError),
    reason = "db_error",
  );
  (StatusCode::INTERNAL_SERVER_ERROR, Json(SignInResponse {
    error_code: Some(SignInError::InternalError),
    jwt_token: None,
    user_data: None,
  }))
}

/*** Handlers ***/

pub async fn handle_sign_in(app_state: State<AppState>, Json(payload): Json<SignInRequest>) -> (StatusCode, Json<SignInResponse>) {
  match payload {
    SignInRequest::PASSWORD { email, password } => handle_password_sign_in(app_state, email, password).await,
    SignInRequest::GOOGLE { id_token } => handle_google_sign_in(app_state, id_token).await,
  }
}

pub async fn handle_jwt_sign_in(State(app_state): State<AppState>, TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>) -> (StatusCode, Json<SignInResponse>) {
  if let Ok(claims) = jwt::get_jwt_claims(&bearer.token(), &app_state.jwt_secret) {
    if let Ok(uuid) = Uuid::parse_str(&claims.sub) {
      let user = user_data::get_user_by_uuid(&app_state.pool, &uuid).await;
      if let Ok(user) = user {
        if let Err(_) = user_data::update_last_seen(&app_state.pool, &uuid).await { 
          return internal_error_response();
        }
        tracing::debug!(
          event = "sign_in_success",
          method = "jwt",
          user_uuid = %uuid,
        );
        return (StatusCode::OK, Json(SignInResponse {
          error_code: None, 
          jwt_token: None,
          user_data: user,
        }));
      }
    }
    return unauthorized_response(SignInError::InvalidCredentials);
  }

  unauthorized_response(SignInError::TokenExpired)
}

/*** Password ***/

async fn handle_password_sign_in(State(app_state): State<AppState>, email: String, password: String) -> (StatusCode, Json<SignInResponse>) {
  // Block more then x sign in attempts in under 5 minutes
  if let Ok(locked) = sign_in_attempts::is_locked(&app_state.redis_pool, &email, app_state.max_sign_in_attempts).await {
    if locked {
      tracing::warn!("Too many sign in attempts!");
      return unauthorized_response(SignInError::InvalidCredentials);
    }
  }

  let user = user_data::get_user_by_email(&app_state.pool, &email).await;

  if let Ok(Some(user)) = user {
    if let Some(password_hash) = &user.password_hash {
      if verify_password(&password, password_hash) {
        let _ = sign_in_attempts::clear_sign_in_attempts(&app_state.redis_pool, &email).await;
        return success_response(&app_state, user, "password").await;
      }
    }
  }
  
  let _ = verify_password(&password, DUMMY_HASH);

  let _ = sign_in_attempts::increament_sign_in_attempts(&app_state.redis_pool, &email, app_state.sign_in_attempts_lock_sec).await;

  // Invalid login
  unauthorized_response(SignInError::InvalidCredentials)
}

/*** Google ***/

async fn handle_google_sign_in(State(app_state): State<AppState>, id_token: String) -> (StatusCode, Json<SignInResponse>) {
  // Getting the claims from google
  tracing::debug!("Received ID token: {}", id_token);
  tracing::debug!("client id: {}", &app_state.google_console_client_id);
  let claims = match get_google_claims(&id_token, &app_state.google_console_client_id).await {
    Ok(claims) => claims,
    Err(_) => {
        return unauthorized_response(SignInError::InvalidCredentials)
      },  
  };

  tracing::debug!("Looking up user by google_sub: {}", claims.sub);
  tracing::debug!("Looking up user by email: {}", claims.email);

  // Try finding user by google_sub
  if let Ok(Some(user)) = user_data::get_user_by_google_sub(&app_state.pool, &claims.sub).await {
    if !claims.email_verified {
      return unauthorized_response(SignInError::NeedToVerifyEmail);
    }
    return success_response(&app_state, user, "google").await;
  }

  // Try linking to existing email. If google sub exist, not auth.
  if let Ok(Some(mut user)) = user_data::get_user_by_email(&app_state.pool, &claims.email).await {
    if user.google_sub.is_none() {
      if let Err(_) = user_data::link_google_sub(&app_state.pool, &user.uuid, &claims.sub).await {
        return internal_error_response();
      }
      if !claims.email_verified {
        return unauthorized_response(SignInError::NeedToVerifyEmail);
      }
      user.google_sub = Some(claims.sub.clone());
      tracing::info!(
        event = "google_sub_link",
        user_uuid = %user.uuid,
      );
      return success_response(&app_state, user, "google").await;
    }
  }
  
  unauthorized_response(SignInError::InvalidCredentials)
}