use uuid::Uuid;
use time::OffsetDateTime;
use serde::{Serialize, Deserialize};
use sqlx::{Error, FromRow, PgPool, query_as};

use crate::auth::db::sign_up_session::SignUpSession;

#[derive(Serialize, Deserialize, FromRow)]
pub struct UserData {
  pub uuid: Uuid,
  pub email: String,
  pub email_verified: bool,
  pub name: String,
  pub password_hash: Option<String>,
  pub google_sub: Option<String>,
  pub phone_num: String,
  pub created_at: OffsetDateTime,
  pub last_seen_at: Option<OffsetDateTime>,
  pub picture: Option<String>,
}

pub async fn get_user_by_uuid(pool: &PgPool, uuid: &Uuid) -> Result<Option<UserData>, Error> {
  let user = query_as!(
    UserData,
    r#"
    SELECT uuid, email, email_verified, name, password_hash, google_sub, phone_num, created_at, last_seen_at, picture
    FROM users
    WHERE uuid = $1
    "#,
    uuid
  )
  .fetch_optional(pool)
  .await?;

  Ok(user)
}

pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<UserData>, Error> {
  let user = query_as!(
    UserData,
    r#"
    SELECT uuid, email, email_verified, name, password_hash, google_sub, phone_num, created_at, last_seen_at, picture
    FROM users
    WHERE email = $1
    "#,
    email
  )
  .fetch_optional(pool)
  .await?;

  Ok(user)
}

pub async fn get_user_by_google_sub(pool: &PgPool, sub: &str) -> Result<Option<UserData>, Error> {
  let user = query_as!(
    UserData,
    r#"
    SELECT uuid, email, email_verified, name, password_hash, google_sub, phone_num, created_at, last_seen_at, picture
    FROM users
    WHERE google_sub = $1
    "#,
    sub
  )
  .fetch_optional(pool)
  .await?;

  Ok(user)
}

pub async fn link_google_sub(pool: &PgPool, uuid: &Uuid, sub: &str) -> Result<(), Error> {
  sqlx::query!("UPDATE users SET google_sub = $1 WHERE uuid = $2", sub, uuid)
    .execute(pool)
    .await
    .map(|_| ())
}

pub async fn update_last_seen(pool: &PgPool, uuid: &Uuid) -> Result<(), Error> {
  sqlx::query!("UPDATE users SET last_seen_at = now() WHERE uuid = $1", uuid)
    .execute(pool)
    .await
    .map(|_| ())
}

pub async fn create_user(pool: &PgPool, session: &SignUpSession) -> Result<UserData, Error> {
  let user = sqlx::query_as!(
    UserData,
    r#"
    INSERT INTO users (uuid, email, email_verified, name, password_hash, google_sub, phone_num, created_at, last_seen_at, picture)
    VALUES ($1, $2, false, $3, $4, $5, $6, now(), now(), $7)
    RETURNING uuid, email, password_hash, email_verified, name, google_sub, phone_num, created_at, last_seen_at, picture
    "#,
    Uuid::new_v4(),
    session.email,
    session.name,
    session.password_hash,
    session.google_sub,
    session.phone_num,
    session.picture,
  )
  .fetch_one(pool)
  .await?;

  Ok(user)
}

// TODO: Add link_picture, verify_email, change_password, update_last_seen, change_email, link_password
