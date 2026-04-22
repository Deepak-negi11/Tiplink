use actix_web::{dev::ServiceRequest, error::ErrorUnauthorized, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::env;

use crate::services::auth::Claim;

/// Validates a JWT Bearer token and injects the user ID (Uuid) into the request extensions.
pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // JWT_SECRET is required — never fall back to an insecure default
    let secret = env::var("JWT_SECRET")
        .expect("FATAL: JWT_SECRET environment variable is not set.");
    let token = credentials.token();

    // Verify token structure, expiration, and signature
    let validation = Validation::default();
    
    match decode::<Claim>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    ) {
        Ok(token_data) => {
            // Store the user configuration payload internally so your handlers can easily access the user's ID
            req.extensions_mut().insert(token_data.claims.sub);
            
            Ok(req)
        }
        Err(_) => {
            // Token is invalid, expired, or improperly signed
            Err((ErrorUnauthorized("Invalid or expired authorization token"), req))
        }
    }
}
