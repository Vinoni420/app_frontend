use serde::{Serialize, Deserialize};
use sqlx::types::Json;
use deadpool_redis::Pool;
use deadpool_redis::redis::AsyncCommands;
use anyhow::Error;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct SmsCode {
  code: String,
  attempts_count: u32,
}

pub async fn store_code(pool: &Pool, uuid: &Uuid, code: &str, expiration_time: u64) -> Result<(), Error> {
  let mut conn = pool.get().await?;
  let key = format!("sms_code:{}", uuid);
  let sms_code = Json(SmsCode {
    code: code.to_string(),
    attempts_count: 0,
  });
  let json_str = serde_json::to_string(&sms_code)?;
  let _: () = conn.set_ex(&key, &json_str, expiration_time).await?;
  Ok(())
}

pub async fn verify_code(pool: &Pool, uuid: &Uuid, code: &str, max_attempts: u32) -> Result<bool, Error> {
  let mut conn = pool.get().await?;
  let key = format!("sms_code:{}", uuid);
  let json_str: String = conn.get(&key).await?;

  let mut code_struct: SmsCode = serde_json::from_str(&json_str)?;

  if code_struct.attempts_count > max_attempts {
    return Ok(false);
  }
  if code_struct.code != code.to_string() {
    // Update the attempts count on DB
    code_struct.attempts_count += 1;
    let json_str = serde_json::to_string(&code_struct)?;
    let _: () = conn.set(&key, &json_str).await?;
    return Ok(false);
  }
  Ok(true)
}

pub async fn get_code_exist(pool: &Pool, uuid: &Uuid) -> Result<(), Error> {
  let mut conn = pool.get().await?;
  let key = format!("sms_code:{}", uuid);
  let json_str: String = conn.get(&key).await?;
  let _: SmsCode = serde_json::from_str(&json_str)?;
  return Ok(());
}

pub async fn get_code_attempts_count(pool: &Pool, uuid: &Uuid) -> Result<u32, Error> {
  let mut conn = pool.get().await?;
  let key = format!("sms_code:{}", uuid);
  let json_str: String = conn.get(&key).await?;
  let code_struct: SmsCode = serde_json::from_str(&json_str)?;
  return Ok(code_struct.attempts_count);
}
