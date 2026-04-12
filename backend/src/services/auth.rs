use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug,Serialize,Deserialize)]
pub struct Claim {
    pub sub :Uuid,
    pub exp: usize,
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error>{
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str , hash: &str)->bool{
    match PasswordHash::new(hash) {
        Ok(parsed_hash) => {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok()
        },
        Err(_) => false,
    }
}

pub fn generate_jwt(user_id:Uuid , secret_key:&str) -> Result<String, jsonwebtoken::errors::Error>{
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("Valid Timestamp")
        .timestamp() as usize;

    let claim = Claim{
        sub :user_id,
        exp : expiration,
    };

    encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(secret_key.as_ref())
    )
}