use axum::{
  extract::{Json, State},
  http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

use crate::app_state::AppState;
use crate::auth::db::sign_up_session;
use crate::auth::db::sms_code;
use crate::api::send_sms::{send_sms_code, SmsCodeSendError};

/*** Json Structs **/

#[derive(Deserialize)]
pub struct SmsRequest {
  uuid: Uuid,
  phone_num: String,
}

#[derive(Deserialize)]
pub struct SmsVerifyRequest {
  uuid: Uuid,
  code: String,
}

#[derive(Serialize)]
pub struct SmsRequestResponse {
  error_code: Option<SmsRequestError>,
}

#[derive(Serialize)]
pub struct SmsVerifyResponse {
  error_code: Option<SmsVerifyError>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SmsRequestError {
  SessionNotFound,
  NeedToWaitBeforeResend,
  PhoneNumNotMatching,
  InternalError,
  InvalidNumber,
  APIError,
  SmsAlreadyVerified,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SmsVerifyError {
  SessionNotFound,
  NeedToResendCode,
  WrongCode,
  InternalError,
  TooManyAttempts,
}

const SMS_MESSAGE: &str = "is your Getly verification code. It will last for 5 minutes";

/*** Helpers ***/

fn success_send_response(uuid: Uuid) -> (StatusCode, Json<SmsRequestResponse>) {
  tracing::info!(
    event = "sign_up_sms_send_success",
    uuid = %uuid,
  );
  (StatusCode::OK, Json(SmsRequestResponse {
    error_code: None,
  }))
}

fn warn_send_response(status_code: StatusCode, error_code: SmsRequestError) -> (StatusCode, Json<SmsRequestResponse>) {
  tracing::warn!(
    event = "sign_up_send_failure",
    error_code = ?(error_code),
  );
  (status_code, Json(SmsRequestResponse {
    error_code: Some(error_code),
  }))
}

fn error_send_response(status_code: StatusCode, error_code: SmsRequestError) -> (StatusCode, Json<SmsRequestResponse>) {
  tracing::error!(
    event = "sign_up_send_failure",
    error_code = ?(error_code),
  );
  (status_code, Json(SmsRequestResponse {
    error_code: Some(error_code),
  }))
}

fn success_verify_response(uuid: Uuid) -> (StatusCode, Json<SmsVerifyResponse>) {
  tracing::info!(
    event = "sign_up_sms_verify_success",
    uuid = %uuid,
  );
  (StatusCode::OK, Json(SmsVerifyResponse {
    error_code: None,
  }))
}

fn warn_verify_response(status_code: StatusCode, error_code: SmsVerifyError) -> (StatusCode, Json<SmsVerifyResponse>) {
  tracing::warn!(
    event = "sign_up_verify_failure",
    error_code = ?(error_code),
  );
  (status_code, Json(SmsVerifyResponse {
    error_code: Some(error_code),
  }))
}

fn error_verify_response(status_code: StatusCode, error_code: SmsVerifyError) -> (StatusCode, Json<SmsVerifyResponse>) {
  tracing::error!(
    event = "sign_up_verify_failure",
    error_code = ?(error_code),
  );
  (status_code, Json(SmsVerifyResponse {
    error_code: Some(error_code),
  }))
}

/*** Handlers ***/

pub async fn handle_sms_request(app_state: State<AppState>, Json(payload): Json<SmsRequest>) -> (StatusCode, Json<SmsRequestResponse>) {
  if let Ok(session) = sign_up_session::get_sign_up_session(&app_state.redis_pool, &payload.uuid).await {
    if session.sms_verified {
      return error_send_response(StatusCode::CONFLICT, SmsRequestError::SmsAlreadyVerified);
    }
    if let Some(phone_num) = session.phone_num {
      if payload.phone_num != phone_num.as_str() {
        return warn_send_response(StatusCode::CONFLICT, SmsRequestError::PhoneNumNotMatching);
      }
      if let Some(last_sent_time) = session.sms_sent_at {
        if Utc::now().timestamp() - last_sent_time > app_state.sms_code_resend_sec {
          if let Err(error) = send_sms_code(&app_state.redis_pool, &payload.uuid, app_state.sms_code_expiration_sec, &app_state.vonage_api_key, &app_state.vonage_api_secret,
              &payload.phone_num, &app_state.company_phone, SMS_MESSAGE).await {
            return match error {
              SmsCodeSendError::APIConnectionError | SmsCodeSendError::APIInternalError | SmsCodeSendError::APIAccountError | SmsCodeSendError::TooManyRequests
                => error_send_response(StatusCode::BAD_GATEWAY, SmsRequestError::APIError),
              SmsCodeSendError::InvalidCreds | SmsCodeSendError::InvalidParams | SmsCodeSendError::DeserializationError | SmsCodeSendError::InternalError | SmsCodeSendError::UnknownError(_)
                => error_send_response(StatusCode::INTERNAL_SERVER_ERROR, SmsRequestError::InternalError),
              SmsCodeSendError::InvalidNumber
                => warn_send_response(StatusCode::UNPROCESSABLE_ENTITY, SmsRequestError::InvalidNumber),
            }
          }
          if let Err(_) = sign_up_session::update_sms_send_time(&app_state.redis_pool, &payload.uuid).await {
            return error_send_response(StatusCode::INTERNAL_SERVER_ERROR, SmsRequestError::InternalError);
          }
          return success_send_response(payload.uuid);
        }
        return warn_send_response(StatusCode::UNAUTHORIZED, SmsRequestError::NeedToWaitBeforeResend);
      }
      return error_send_response(StatusCode::INTERNAL_SERVER_ERROR, SmsRequestError::InternalError);
    } else {
      if let Err(_) = sign_up_session::link_phone_num(&app_state.redis_pool, &payload.uuid, &payload.phone_num).await {
        return error_send_response(StatusCode::INTERNAL_SERVER_ERROR, SmsRequestError::InternalError);
      }
      if let Err(error) = send_sms_code(&app_state.redis_pool, &payload.uuid, app_state.sms_code_expiration_sec, &app_state.vonage_api_key, &app_state.vonage_api_secret,
          &payload.phone_num, &app_state.company_phone, SMS_MESSAGE).await {
        return match error {
          SmsCodeSendError::APIConnectionError | SmsCodeSendError::APIInternalError | SmsCodeSendError::APIAccountError | SmsCodeSendError::TooManyRequests
            => error_send_response(StatusCode::BAD_GATEWAY, SmsRequestError::APIError),
          SmsCodeSendError::InvalidCreds | SmsCodeSendError::InvalidParams | SmsCodeSendError::DeserializationError | SmsCodeSendError::InternalError | SmsCodeSendError::UnknownError(_)
            => error_send_response(StatusCode::INTERNAL_SERVER_ERROR, SmsRequestError::InternalError),
          SmsCodeSendError::InvalidNumber
            => warn_send_response(StatusCode::UNPROCESSABLE_ENTITY, SmsRequestError::InvalidNumber),
        }
      }
      if let Err(_) = sign_up_session::update_sms_send_time(&app_state.redis_pool, &payload.uuid).await {
        return error_send_response(StatusCode::INTERNAL_SERVER_ERROR, SmsRequestError::InternalError);
      }
      return success_send_response(payload.uuid);
    }
  }
  warn_send_response(StatusCode::UNAUTHORIZED, SmsRequestError::SessionNotFound)
}

pub async fn handle_sms_verify(app_state: State<AppState>, Json(payload): Json<SmsVerifyRequest>) -> (StatusCode, Json<SmsVerifyResponse>) {
  if let Ok(_) = sign_up_session::get_sign_up_session(&app_state.redis_pool, &payload.uuid).await {
    if let Err(_) = sms_code::get_code_exist(&app_state.redis_pool, &payload.uuid).await {
      return warn_verify_response(StatusCode::GONE, SmsVerifyError::NeedToResendCode);
    }
    if let Ok(correct) = sms_code::verify_code(&app_state.redis_pool, &payload.uuid, &payload.code, app_state.sms_code_max_attemps).await {
      if correct {
        if let Err(_) = sign_up_session::verify_sms(&app_state.redis_pool, &payload.uuid).await {
          return error_verify_response(StatusCode::INTERNAL_SERVER_ERROR, SmsVerifyError::InternalError);
        }
        return success_verify_response(payload.uuid);
      }
      if let Ok(attempts) = sms_code::get_code_attempts_count(&app_state.redis_pool, &payload.uuid).await {
        if attempts > app_state.sms_code_max_attemps {
          return warn_verify_response(StatusCode::UNAUTHORIZED, SmsVerifyError::TooManyAttempts);
        } else {
          return warn_verify_response(StatusCode::UNAUTHORIZED, SmsVerifyError::WrongCode);
        }
      }
      return error_verify_response(StatusCode::INTERNAL_SERVER_ERROR, SmsVerifyError::InternalError);
    }
    return error_verify_response(StatusCode::INTERNAL_SERVER_ERROR, SmsVerifyError::InternalError);
  }
  warn_verify_response(StatusCode::UNAUTHORIZED, SmsVerifyError::SessionNotFound)
}
