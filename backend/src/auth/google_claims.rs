use google_jwt_verify::{ Client, Error };
use tracing; // Make sure you have tracing or another logging crate setup
use chrono::prelude::*;

pub struct GoogleClaims {
    pub sub: String,
    pub email: String,
    pub email_verified: bool,
    pub name: String,
    pub picture: Option<String>,
    
}

pub async fn get_google_claims(token_id: &str, client_id: &str) -> Result<GoogleClaims, Error> {
    let client = Client::new(client_id);
    let local_now = Local::now();
    println!("Current local time: {}", local_now);
    
    tracing::info!("Using Google client ID: {}", client_id);
    tracing::info!("token is: {}", token_id);

    let token = match client.verify_id_token(token_id) {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("Failed to verify Google ID token: {:?}", e);   
            return Err(e);
        }
    };

    let sub = token.get_claims().get_subject();
    let email = token.get_payload().get_email();
    let name = token.get_payload().get_name();
    let email_verified = token.get_payload().is_email_verified();

    tracing::info!(email, name);

    let picture_raw = token.get_payload().get_picture_url();
    let picture = if picture_raw.is_empty() {
        None
    } else {
        Some(picture_raw)
    };

    Ok(GoogleClaims {
        sub,
        email,
        name,
        email_verified,
        picture,
    })
}