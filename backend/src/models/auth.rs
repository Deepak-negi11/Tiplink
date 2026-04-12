use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// What Next.js sends to `POST /signup`
#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SigninRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String, 
    pub refresh_token: Option<String>,
    pub user_id: Uuid,
    pub email: String,
    pub public_key: String, 
}