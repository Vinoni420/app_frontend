mod auth;
mod api;
mod app_state;
mod ping;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};
use dotenv::dotenv;
use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*};

use crate::auth::sign_in;
use crate::auth::sign_up_start;
use crate::auth::sign_up_sms;
use crate::auth::sign_up_complete;
use crate::app_state::create_app_state;

#[tokio::main]
async fn main() {
  // Load .env
  dotenv().ok();

  tracing_subscriber::registry()
    .with(fmt::layer())
    .with(EnvFilter::from_default_env())
    .init();

  let app_state = create_app_state().await;

  // Allow all origins for dev
  let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

  let app = Router::new()
    .route("/auth/sign-in", post(sign_in::handle_sign_in))
    .route("/auth/me", get(sign_in::handle_jwt_sign_in))
    .route("/ping", get(ping::ping_handler))
    .route("/auth/sign-up/start", post(sign_up_start::handle_start))
    .route("/auth/sign-up/send-sms", post(sign_up_sms::handle_sms_request))
    .route("/auth/sign-up/verify-sms", post(sign_up_sms::handle_sms_verify))
    .route("/auth/sign-up/complete", post(sign_up_complete::handle_complete))
    .layer(cors)
    .with_state(app_state);

  let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
  println!("Listening on http://{}", addr);

  axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app.into_make_service())
    .await.unwrap();
}

