use reqwest::Client;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use rand::Rng;
use deadpool_redis::Pool;

use crate::auth::db::sms_code;

#[derive(Serialize)]
struct SmsAPIRequest {
    api_key: String,
    api_secret: String,
    to: String,
    from: String,
    text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SmsCodeSendError {
    APIConnectionError,
    APIInternalError,
    APIAccountError,
    InvalidCreds,
    TooManyRequests,
    InvalidParams,
    InvalidNumber,
    DeserializationError,
    InternalError,
    UnknownError(String),
}

#[derive(Deserialize)]
struct SmsAPIResponse {
    messages: Vec<SmsAPIMessageStatus>,
}

#[derive(Deserialize)]
struct SmsAPIMessageStatus {
    to: String,
    status: String,
    error_text: Option<String>,
}

fn generate_sms_code() -> String {
  let mut rng = rand::rng();
  let code: u32 = rng.random_range(0..1_000_000);
  format!("{:06}", code)
}

pub async fn send_sms_code(pool: &Pool, uuid: &Uuid, expiration_time: u64, api_key: &str, api_secret: &str, to: &str, from: &str, text: &str) -> Result<(), SmsCodeSendError> {
  let code: String = generate_sms_code();
  if let Err(_) = sms_code::store_code(pool, uuid, &code, expiration_time).await {
    tracing::error!(
      event = "send_sms_code_internal_failure",
      uuid = %uuid,
    );
    return Err(SmsCodeSendError::InternalError);
  }

  let text: String = code + " " + text;
    let sms = SmsAPIRequest {
      api_key: api_key.to_string(),
      api_secret: api_secret.to_string(),
      to: to.to_string(),
      from: from.to_string(),
      text: text,
  };

  let client = Client::new();
  let res = client
    .post("https://rest.nexmo.com/sms/json")
    .json(&sms)
    .send()
    .await
    .map_err(|_| SmsCodeSendError::APIConnectionError)?;

  let status = res.status();
  if !status.is_success() {
      tracing::error!(
        event = "send_sms_code_http_failure",
        http_status = status.as_u16(),
        uuid = %uuid,
      );
      return Err(SmsCodeSendError::APIConnectionError);
  }

  let res_body: SmsAPIResponse = res.json().await.map_err(|_| SmsCodeSendError::DeserializationError)?;

  for msg in res_body.messages {
    if msg.status != "0" {
      let error = match msg.status.as_str() {
        "5" => SmsCodeSendError::APIInternalError,
        "1" | "10" => SmsCodeSendError::TooManyRequests,
        "14" | "32"  => SmsCodeSendError::InvalidCreds,
        "7" | "33" => SmsCodeSendError::InvalidNumber,
        "8" | "9" | "11" | "29" => SmsCodeSendError::APIAccountError,
        "2" | "3" | "6" | "12" | "15" | "22" | "23" => SmsCodeSendError::InvalidParams,
        _ => SmsCodeSendError::UnknownError(msg.status.clone()),
      };

      tracing::warn!(
        event = "send_sms_code_failure",
        uuid = %uuid,
        to = msg.to,
        status = msg.status,
        error_text = msg.error_text.as_deref().unwrap_or(""),
      );

      return Err(error);
    }
  }

  Ok(())
}
