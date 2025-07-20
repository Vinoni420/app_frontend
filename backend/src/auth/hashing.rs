use argon2::{Argon2, PasswordHash, PasswordVerifier, PasswordHasher};
use argon2::password_hash::{SaltString, rand_core::OsRng, Error};

pub fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;

    Ok(password_hash.to_string())
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
  match PasswordHash::new(password_hash) {
    Ok(parsed_hash) => Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok(),
    Err(_) => false,
  }
}
