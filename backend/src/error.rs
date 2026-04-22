use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    DatabaseError(diesel::result::Error),
    ExternalApi(String),
    Unauthorized(String),
    BadRequest(String),
    InternalServerError(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DatabaseError(err) => write!(f, "Database error: {}", err),
            AppError::ExternalApi(err) => write!(f, "External API error: {}", err),
            AppError::Unauthorized(err) => write!(f, "Unauthorized: {}", err),
            AppError::BadRequest(err) => write!(f, "Bad request: {}", err),
            AppError::InternalServerError(err) => write!(f, "Internal server error: {}", err),
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::DatabaseError(diesel::result::Error::NotFound) => StatusCode::NOT_FOUND,
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ExternalApi(_) => StatusCode::BAD_GATEWAY,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let error_msg = match self {
            AppError::DatabaseError(diesel::result::Error::NotFound) => "Not found".to_string(),
            AppError::DatabaseError(_) => "Database error".to_string(),
            AppError::ExternalApi(err) => err.clone(),
            AppError::Unauthorized(err) => err.clone(),
            AppError::BadRequest(err) => err.clone(),
            AppError::InternalServerError(err) => err.clone(),
        };

        HttpResponse::build(self.status_code()).json(ErrorResponse { error: error_msg })
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<r2d2::Error> for AppError {
    fn from(err: r2d2::Error) -> Self {
        AppError::InternalServerError(format!("Pool error: {}", err))
    }
}

impl From<Box<dyn std::error::Error>> for AppError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}
