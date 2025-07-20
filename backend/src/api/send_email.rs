use axum::http::StatusCode;
use resend_rs::{Resend, Error, types::CreateEmailBaseOptions};

pub async fn send_email(resend: &Resend, from: &str, to: Vec<&str>, subject: &str, html_body: &str) -> Result<(), Error> {
  let email = CreateEmailBaseOptions::new(from, to, subject).with_html(html_body);
  if let Err(error) = resend.emails.send(email).await {
    tracing::error!(
      event = "send_email_failure",
      error = %error,
    );
    return Err(error);
  }
  Ok(())
}
