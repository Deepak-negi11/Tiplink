use actix_web::{HttpResponse, http::StatusCode};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MpcError {
    #[error("Unauthorised: {0}")]
    Unauthorised(String),
    #[error("Not Found: {0}")]
    NotFound(String),
    #[error("Security Violation: {0}")]
    SecurityViolation(String),
    #[error("Crypto error: {0}")]
    Crypto(String),
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Internal: {0}")]
    Internal(String),
}

impl actix_web::ResponseError for MpcError {
    fn status_code(&self) -> StatusCode {
        match self {
            MpcError::Unauthorised(_) => StatusCode::UNAUTHORIZED,
            MpcError::NotFound(_) => StatusCode::NOT_FOUND,
            MpcError::SecurityViolation(_) => StatusCode::FORBIDDEN,
            MpcError::BadRequest(_) => StatusCode::BAD_REQUEST,
            MpcError::Crypto(_) => StatusCode::INTERNAL_SERVER_ERROR,
            MpcError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(serde_json::json!({
            "error": self.to_string()
        }))
    }
}

impl From<frost_ed25519::Error> for MpcError {
    fn from(err: frost_ed25519::Error) -> Self {
        MpcError::Crypto(format!("FROST cryptographic error: {}", err))
    }
}