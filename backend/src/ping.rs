use axum::http::StatusCode;

pub async fn ping_handler() -> StatusCode {
  StatusCode::OK
}
