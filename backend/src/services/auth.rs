use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, decode, DecodingKey, EncodingKey, Header, Validation};
use rand::{distributions::Alphanumeric, Rng};
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claim {
    pub sub: Uuid,
    pub exp: usize,
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed_hash) => Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok(),
        Err(_) => false,
    }
}

pub fn generate_access_token(user_id: Uuid, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(1))
        .expect("Valid Timestamp")
        .timestamp() as usize;

    let claim = Claim {
        sub: user_id,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(secret.as_bytes())
    )
}

pub fn generate_refresh_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

pub fn verify_access_token(token: &str, secret: &str) -> Result<Claim, jsonwebtoken::errors::Error> {
    let validation = Validation::default();
    let decoded = decode::<Claim>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation
    )?;
    Ok(decoded.claims)
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}