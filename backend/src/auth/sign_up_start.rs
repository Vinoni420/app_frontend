use axum::{
  extract::{Json, State},
  http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::hashing::hash_password;
use crate::auth::google_claims::get_google_claims;
use crate::auth::captcha::verify_recaptcha;
use crate::auth::db::user_data;
use crate::auth::db::sign_up_session;
use crate::app_state::AppState;

#[derive(Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum StartRequest {
  PASSWORD { email: String, password: String, name: String, captcha_token: String },
  GOOGLE { id_token: String },
}

#[derive(Serialize)]
pub struct StartResponse {
  sign_up_token: Option<Uuid>,
  error_code: Option<StartError>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StartError {
  CaptchaVerificationFailed,
  EmailAlreadyExists,
  InvalidToken,
  InternalError,
}

/*** Helpers ***/

fn success_response(uuid: Uuid) -> (StatusCode, Json<StartResponse>) {
  tracing::info!(
    event = "sign_up_start_success",
    uuid = %uuid,
  );
  (StatusCode::OK, Json(StartResponse {
    sign_up_token: Some(uuid),
    error_code: None,
  }))
}

fn warn_response(status_code: StatusCode, error_code: StartError, email: &str, reason: &str) -> (StatusCode, Json<StartResponse>) {
  tracing::warn!(
    event = "sign_up_start_failure",
    email = email,
    error_code = ?(error_code),
    reason = reason,
  );
  (status_code, Json(StartResponse {
    sign_up_token: None,
    error_code: Some(error_code),
  }))
}

fn error_response(status_code: StatusCode, error_code: StartError, email: &str, reason: &str) -> (StatusCode, Json<StartResponse>) {
  tracing::error!(
    event = "sign_up_start_failure",
    email = email,
    error_code = ?(error_code),
    reason = reason,
  );
  (status_code, Json(StartResponse {
    sign_up_token: None,
    error_code: Some(error_code),
  }))
}

fn internal_error_response(email: &str, reason: &str) -> (StatusCode, Json<StartResponse>) {
  error_response(StatusCode::INTERNAL_SERVER_ERROR, StartError::InternalError, email, reason)
}

/*** Handlers ***/

pub async fn handle_start(app_state: State<AppState>, Json(payload): Json<StartRequest>) -> (StatusCode, Json<StartResponse>) {
  match payload {
    StartRequest::PASSWORD { email, password, name, captcha_token } => handle_password_start(app_state, email, password, name, captcha_token).await,
    StartRequest::GOOGLE { id_token } => handle_google_start(app_state, id_token).await,
  }
}

async fn handle_password_start(
    State(app_state): State<AppState>,
    email: String,
    password: String,
    name: String,
    captcha_token: String,
) -> (StatusCode, Json<StartResponse>) {
    // 1. Verify CAPTCHA
    if let Ok(passed) = verify_recaptcha(&captcha_token, &app_state.captcha_secret_key).await {
        if !passed {
            return warn_response(
                StatusCode::UNAUTHORIZED,
                StartError::CaptchaVerificationFailed,
                &email,
                "captcha_test_failed",
            );
        }

        // 2. Check if user already exists
        match user_data::get_user_by_email(&app_state.pool, &email).await {
            Ok(Some(_)) => {
                return warn_response(
                    StatusCode::CONFLICT,
                    StartError::EmailAlreadyExists,
                    &email,
                    "sign_up_attempt_with_existing_email",
                );
            }
            Ok(None) => {
                // user not found, continue
            }
            Err(e) => {
                return internal_error_response(&email, &format!("db_error: {e}"));
            }
        }

        // 3. Hash password
        if let Ok(hashed_password) = hash_password(&password) {
            // 4. Start sign-up session in Redis
            if let Ok(uuid) = sign_up_session::start_sign_up_password(
                &app_state.redis_pool,
                &email,
                &hashed_password,
                &name,
                app_state.sign_up_session_expiration_sec,
            )
            .await
            {
                return success_response(uuid);
            }

            return internal_error_response(&email, "cannot_connect_to_redis");
        }

        return internal_error_response(&email, "argon2_password_hashing_failed");
    }

    error_response(
        StatusCode::BAD_GATEWAY,
        StartError::CaptchaVerificationFailed,
        &email,
        "unable_to_connect_to_google_servers",
    )
}


async fn handle_google_start(State(app_state): State<AppState>, id_token: String) -> (StatusCode, Json<StartResponse>) {
  if let Ok(claims) = get_google_claims(&id_token, &app_state.google_console_client_id).await {
    if let Ok(_) = user_data::get_user_by_email(&app_state.pool, &claims.email).await {
      return warn_response(StatusCode::CONFLICT, StartError::EmailAlreadyExists, &claims.email, "sign_up_attempt_with_existing_email");
    }
    if let Ok(uuid) = sign_up_session::start_sign_up_google(&app_state.redis_pool, &claims, app_state.sign_up_session_expiration_sec).await {
      return success_response(uuid);
    }
    return internal_error_response(&claims.email, "cannot_connect_to_redis");
  }
  error_response(StatusCode::UNAUTHORIZED, StartError::InvalidToken, "None", "token_is_invalid")
}