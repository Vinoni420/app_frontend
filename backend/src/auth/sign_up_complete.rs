use axum::{
  extract::{Json, State},
  http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth::jwt;
use crate::auth::db::sign_up_session;
use crate::auth::db::user_data::{self, UserData};

#[derive(Deserialize)]
pub struct Request {
  uuid: Uuid,
}

#[derive(Serialize)]
pub struct Response {
  jwt_token: Option<String>,
  user_data: Option<UserData>,
  error_code: Option<Error>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Error {
  SessionNotFound,
  EmailNotVerified,
  CodeNotVerified,
  InternalError,
}

fn internal_error_response(uuid: Uuid) -> (StatusCode, Json<Response>) {
  tracing::error!(
    event = "sign_up_complete_failure",
    uuid = %uuid,
    error_code = ?Error::InternalError,
  );
  (StatusCode::INTERNAL_SERVER_ERROR, Json(Response {
    jwt_token: None,
    user_data: None,
    error_code: Some(Error::InternalError),
  }))
}

pub async fn handle_complete(State(app_state): State<AppState>, Json(payload): Json<Request>) -> (StatusCode, Json<Response>) {
  if let Ok(session) = sign_up_session::get_sign_up_session(&app_state.redis_pool, &payload.uuid).await {
    if !session.sms_verified {
      tracing::warn!(
        event = "sign_up_complete_failure",
        uuid = %payload.uuid,
        error_code = ?Error::CodeNotVerified,
      );
      return (StatusCode::UNAUTHORIZED, Json(Response {
        jwt_token: None,
        user_data: None,
        error_code: Some(Error::CodeNotVerified),
      }));
    }
    if let Err(_) = sign_up_session::delete_session(&app_state.redis_pool, &payload.uuid).await {
      return internal_error_response(payload.uuid);
    }
    if let Ok(user_data) = user_data::create_user(&app_state.pool, &session).await {
      if !user_data.email_verified {
        tracing::warn!(
          event = "sign_up_complete_failure",
          uuid = %payload.uuid,
          error_code = ?Error::EmailNotVerified,
        );
        return (StatusCode::UNAUTHORIZED, Json(Response {
          jwt_token: None,
          user_data: None,
          error_code: Some(Error::EmailNotVerified),
        }));
      }
      if let Ok(jwt_token) = jwt::create_jwt_token(&user_data.uuid, &app_state.jwt_secret, app_state.jwt_expiration_days)
      {
        tracing::info!(
          event = "sign_up_complete_success",
          uuid = %payload.uuid,
        );
        return (StatusCode::OK, Json(Response {
          jwt_token: Some(jwt_token),
          user_data: Some(user_data),
          error_code: None,
        }));
      }
      return internal_error_response(payload.uuid);
    }
    return internal_error_response(payload.uuid);
  }

  tracing::warn!(
    event = "sign_up_complete_failure",
    uuid = %payload.uuid,
    error_code = ?Error::SessionNotFound,
  );
  (StatusCode::UNAUTHORIZED, Json(Response {
    jwt_token: None,
    user_data: None,
    error_code: Some(Error::SessionNotFound),
  }))
}
