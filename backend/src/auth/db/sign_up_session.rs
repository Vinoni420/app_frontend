use sqlx::types::Json;
use serde::{Serialize, Deserialize};
use deadpool_redis::Pool;
use deadpool_redis::redis::AsyncCommands;
use anyhow::Error;
use uuid::Uuid;
use chrono::Utc;

use crate::auth::google_claims::GoogleClaims;

#[derive(Serialize, Deserialize)]
pub struct SignUpSession {
  pub email: String,
  pub name: String,
  pub password_hash: Option<String>,
  pub google_sub: Option<String>,
  pub picture: Option<String>,
  pub phone_num: Option<String>,
  pub sms_sent_at: Option<i64>,
  pub sms_verified: bool,
}

pub async fn start_sign_up_password(pool: &Pool, email: &str, password_hash: &str, name: &str, expiration_time: u64) -> Result<Uuid, Error> {
  let mut conn = pool.get().await?;
  let uuid = Uuid::new_v4();
  let key = format!("sign_up_session:{}", uuid);
  let json = Json(SignUpSession {
    email: email.to_string(),
    name: name.to_string(),
    password_hash: Some(password_hash.to_string()),
    google_sub: None,
    picture: None,
    phone_num: None,
    sms_sent_at: None,
    sms_verified: false,
  });
  let _: () = conn.set_ex(key, serde_json::to_string(&json)?, expiration_time).await?;
  Ok(uuid)
}

pub async fn start_sign_up_google(pool: &Pool, claims: &GoogleClaims, expiration_time: u64) -> Result<Uuid, Error> {
  let mut conn = pool.get().await?;
  let uuid = Uuid::new_v4();
  let key = format!("sign_up_session:{}", uuid);
  let json = Json(SignUpSession {
    email: claims.email.clone(),
    name: claims.name.clone(),
    password_hash: None,
    google_sub: Some(claims.sub.to_string()),
    picture: claims.picture.clone(),
    phone_num: None,
    sms_sent_at: None,
    sms_verified: false,
  });
  let json_str = serde_json::to_string(&json)?;
  let _: () = conn.set_ex(&key, &json_str, expiration_time).await?;
  Ok(uuid)
}

pub async fn get_sign_up_session(pool: &Pool, uuid: &Uuid) -> Result<SignUpSession, Error> {
  let mut conn = pool.get().await?;
  let key = format!("sign_up_session:{}", uuid);
  let json_str: String = conn.get(&key).await?;
  Ok(serde_json::from_str(&json_str)?)
}

pub async fn update_sms_send_time(pool: &Pool, uuid: &Uuid) -> Result<(), Error> {
  let mut conn = pool.get().await?;
  let key = format!("sign_up_session:{}", uuid);
  let json_str: String = conn.get(&key).await?;
  let mut sign_up_session: SignUpSession  = serde_json::from_str(&json_str)?;
  sign_up_session.sms_sent_at = Some(Utc::now().timestamp());
  let json_str = serde_json::to_string(&sign_up_session)?;
  let _: () = conn.set(&key, &json_str).await?;
  Ok(())
}

pub async fn link_phone_num(pool: &Pool, uuid: &Uuid, phone_num: &str) -> Result<(), Error> {
  let mut conn = pool.get().await?;
  let key = format!("sign_up_session:{}", uuid);
  let json_str: String = conn.get(&key).await?;
  let mut sign_up_session: SignUpSession  = serde_json::from_str(&json_str)?;
  sign_up_session.phone_num = Some(phone_num.to_string());
  let json_str = serde_json::to_string(&sign_up_session)?;
  let _: () = conn.set(&key, &json_str).await?;
  Ok(())
}

pub async fn verify_sms(pool: &Pool, uuid: &Uuid) -> Result<(), Error> {
  let mut conn = pool.get().await?;
  let key = format!("sign_up_session:{}", uuid);
  let json_str: String = conn.get(&key).await?;
  let mut sign_up_session: SignUpSession  = serde_json::from_str(&json_str)?;
  sign_up_session.sms_verified = true;
  let json_str = serde_json::to_string(&sign_up_session)?;
  let _: () = conn.set(&key, &json_str).await?;
  Ok(())
}

pub async fn delete_session(pool: &Pool, uuid: &Uuid) -> Result<(), Error> {
  let mut conn = pool.get().await?;
  let key = format!("sign_up_session:{}", uuid);
  let _: () = conn.del(&key).await?;
  Ok(())
}
