use reqwest::{Client, Error};
use serde::Deserialize;

#[derive(Deserialize)]
struct RecaptchaResponse {
    success: bool,
}

pub async fn verify_recaptcha(token: &str, secret_key: &str) -> Result<bool, Error> {
  let client = Client::new();
  let params = vec![
    ("secret", secret_key),
    ("response", token),
  ];

  let response = client.post("https://www.google.com/recaptcha/api/siteverify")
    .form(&params).send().await?.json::<RecaptchaResponse>().await?;

  Ok(response.success)
}
