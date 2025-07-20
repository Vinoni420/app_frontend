use deadpool_redis::Pool;
use deadpool_redis::redis::AsyncCommands;
use anyhow::Error;

pub async fn increament_sign_in_attempts(pool: &Pool, email: &str, expirarion_time: i64) -> Result<(), Error> {
  let mut conn = pool.get().await?;
  let key = format!("sign_in_attempts:{}", email);
  let attempts: u32 = conn.incr(&key, 1).await?;

  // Set expiration on first inc
  if attempts == 1 {
    let _: u32 = conn.expire(&key, expirarion_time).await?;
  }

  Ok(())
}

pub async fn is_locked(pool: &Pool, email: &str, max_attempts: u32) -> Result<bool, Error> {
  let mut conn = pool.get().await?;
  let key = format!("sign_in_attempts:{}", email);
  let attempts: u32 = conn.get(&key).await?;
  Ok(attempts >= max_attempts)
}

pub async fn clear_sign_in_attempts(pool: &Pool, email: &str) -> Result<(), Error> {
  let mut conn = pool.get().await?;
  let key = format!("sign_in_attempts:{}", email);
  let _: u32 = conn.del(&key).await?;
  Ok(())
}
