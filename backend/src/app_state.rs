use std::env::var;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use deadpool_redis::{Config, Pool, Runtime};
use resend_rs::Resend;

#[derive(Clone)]
pub struct AppState {
  pub pool: PgPool,
  pub redis_pool: Pool, 
  pub resend: Resend,
  pub jwt_secret: String,
  pub google_console_client_id: String,
  pub captcha_secret_key: String,
  pub vonage_api_key: String,
  pub vonage_api_secret: String,
  pub company_phone: String,
  pub jwt_expiration_days: i64,
  pub max_sign_in_attempts: u32,
  pub sign_in_attempts_lock_sec: i64,
  pub sign_up_session_expiration_sec: u64,
  pub sms_code_expiration_sec: u64,
  pub sms_code_resend_sec: i64,
  pub sms_code_max_attemps: u32,
}

pub async fn create_app_state() -> AppState {
  AppState {
    pool: create_pg_pool().await,
    redis_pool: create_redis_pool().await,
    resend: Resend::new(&var("RESEND_API_KEY").expect("RESEND_API_KEY var must be set")),
    jwt_secret: var("JWT_SECRET").expect("JWT_SECRET var must be set"),
    google_console_client_id: var("GOOGLE_CONSOLE_CLIENT_ID").expect("GOOGLE_CONSOLE_CLIENT_ID var must be set"),
    captcha_secret_key: var("CAPTCHA_SECRET_KEY").expect("CAPTCHA_SECRET_KEY var must be set"),  
    vonage_api_key: var("VONAGE_API_KEY").expect("VONAGE_API_KEY var must be set"),  
    vonage_api_secret: var("VONAGE_API_SECRET").expect("VONAGE_API_SECRET var must be set"),
    company_phone: "972585339500".to_string(),
    jwt_expiration_days: 30,
    max_sign_in_attempts: 10,
    sign_in_attempts_lock_sec: 300,
    sign_up_session_expiration_sec: 900,
    sms_code_expiration_sec: 300,
    sms_code_resend_sec: 180,
    sms_code_max_attemps: 5,
  }
}

async fn create_pg_pool() -> PgPool {
  let url = var("DATABASE_URL").expect("DATABASE_URL var must be set");
  PgPoolOptions::new().max_connections(5).connect(&url).await.expect("Failed to connect to the database")
}

async fn create_redis_pool() -> Pool {
  let url = var("REDIS_URL").expect("REDIS_URL var must be set");
  let config = Config::from_url(&url);
  config.create_pool(Some(Runtime::Tokio1)).expect("Unable to create redis connection pool")
}
