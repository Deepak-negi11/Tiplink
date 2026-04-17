use actix_web::HttpResponse;
use thiserror::Error;

#[derive(Debug , Error)]
pub enum MpcError{
    #[error("Unauthorised:{0}")]
    Unauthorised(String),
    #[error("Not Found: {0}")]
    NotFound(String),
    #[error("Security Violation:{0}")]
    SecurityViolation(String),
    #[error("Crypto error: {0}")]
    Crypto(String),
    #[error("Bad Request:{0}")]
    BadRequest(String),
    #[error("Internal:{0}")]
    Internal(String)
}


impl actix_web::ResponseError for MpcError{
    fn error_response(&self) -> HttpResponse{
        match self {
            MpcError::Unauthorised(_) => HttpResponse::Unauthorised().json(self.to_string()),
            MpcError::NotFound(_) => HttpResponse::NotFound().json(self.to_string()),
            MpcError::SecurityViolation(_) => HttpResponse::Forbidden().json(self.to_string()),
            MpcError::BadRequest(_) => HttpResponse::BadRequest().json(self.to_string()),
        }
    }
}