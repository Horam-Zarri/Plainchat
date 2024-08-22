use anyhow::Context;

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;

use crate::error::{AppError, Result};

pub async fn hash_password(pass: String) -> Result<String> {
    Ok(tokio::task::spawn_blocking(move || -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let hashed = Argon2::default().hash_password(pass.as_bytes(), &salt)
            .map_err(anyhow::Error::msg)?.to_string();
        Ok(hashed)
    }).await.context("panic while generating hash")??)
}

pub async fn verify_password(pass: String, hashed: String) -> Result<()> {
    Ok(tokio::task::spawn_blocking(move || -> Result<()> {
        let parsed_hash = PasswordHash::new(&hashed)
            .map_err(anyhow::Error::msg)?;
        Argon2::default().verify_password(pass.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::WrongCredentials(Some("Password is incorrect".to_string())))
    }).await.context("panic while verifying password")??)
}