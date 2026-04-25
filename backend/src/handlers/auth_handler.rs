use actix_web::{web, HttpResponse, HttpMessage, HttpRequest};
use uuid::Uuid;
use chrono::{Utc, TimeDelta};
use crate::models::auth::{SignupRequest, SigninRequest, RefreshRequest, LogoutRequest, AuthResponse};
use crate::db::conn::DbPool;
use crate::db::user::User;
use crate::db::session::Session;
use crate::services::auth::{hash_password, verify_password, generate_access_token, generate_refresh_token, hash_token};
use crate::services::dkg::{generate_keypair, Config};
use crate::error::AppError;

/// Returns the JWT secret from env. Panics at startup if not set — never use a default.
fn jwt_secret() -> String {
    std::env::var("JWT_SECRET")
        .expect("FATAL: JWT_SECRET environment variable is not set. Refusing to start with an insecure default.")
}

/// Builds MPC node configuration from environment variables. All node URLs are required.
fn mpc_config() -> Result<Config, AppError> {
    let aws = std::env::var("MPC_NODE_1")
        .map_err(|_| AppError::InternalServerError("MPC_NODE_1 not configured".into()))?;
    let do_ocean = std::env::var("MPC_NODE_2")
        .map_err(|_| AppError::InternalServerError("MPC_NODE_2 not configured".into()))?;
    let cloudflare = std::env::var("MPC_NODE_3")
        .map_err(|_| AppError::InternalServerError("MPC_NODE_3 not configured".into()))?;
    let api_keys = std::env::var("INTERNAL_MPC_KEY").unwrap_or_default();

    Ok(Config { aws, do_ocean, cloudflare, api_keys })
}

pub async fn signup(
    pool: web::Data<DbPool>,
    req: web::Json<SignupRequest>
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".to_string()))?;

    if User::exists_by_email(&mut conn, &req.email)? {
        return Err(AppError::BadRequest("User already exists".to_string()));
    }

    let hashed = hash_password(&req.password).map_err(|_| AppError::InternalServerError("Hashing failed".to_string()))?;
    let user_id = Uuid::new_v4(); 

    let dkg_conf = mpc_config()?;
    let public_key = generate_keypair(&dkg_conf, user_id).await
        .map_err(|e| {
            let err_msg = format!("Distributed key generation failed. MPC nodes may be offline: {}", e);
            println!("SIGNUP ERROR: {}", err_msg);
            AppError::InternalServerError(err_msg)
        })?;
    
    let user = User::signup(user_id, &mut conn, &req.email, &hashed, &public_key)?;
    
    let secret = jwt_secret();
    let token = generate_access_token(user.id, &secret)
        .map_err(|_| AppError::InternalServerError("Token generation failed".to_string()))?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        refresh_token: None,
        user_id: user.id,
        email: user.email,
        public_key: user.public_key,
    }))
}

pub async fn signin(
    pool: web::Data<DbPool>,
    req: web::Json<SigninRequest>
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".to_string()))?;

    let user = User::find_by_email(&mut conn, &req.email)?
        .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    if !verify_password(&req.password, &user.password) {
         return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    if !user.is_active {
        return Err(AppError::Unauthorized("Account is deactivated".to_string()));
    }

    let secret = jwt_secret();
    let token = generate_access_token(user.id, &secret)
        .map_err(|_| AppError::InternalServerError("Token generation failed".to_string()))?;
        
    let refresh_token_raw = generate_refresh_token();
    let hashed_refresh = hash_token(&refresh_token_raw);
    
    let expires = Utc::now() + TimeDelta::try_days(7).unwrap_or_default();
    let new_session = crate::db::session::NewSession {
        id: Uuid::new_v4(),
        user_id: user.id,
        refresh_token: &hashed_refresh,
        device_info: None,
        ip_address: None,
        expires_at: expires,
    };
    Session::create_session(&mut conn, new_session)?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        refresh_token: Some(refresh_token_raw),
        user_id: user.id,
        email: user.email,
        public_key: user.public_key,
    }))
}

pub async fn refresh(
    pool: web::Data<DbPool>,
    req: web::Json<RefreshRequest>
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".to_string()))?;
    
    let hashed_incoming = hash_token(&req.refresh_token);
    
    let session = Session::find_valid_by_token(&mut conn, &hashed_incoming)?
        .ok_or_else(|| AppError::Unauthorized("Invalid session".to_string()))?;
        
    if session.revoked_at.is_some() || session.expires_at < Utc::now() {
        return Err(AppError::Unauthorized("Session expired or revoked".to_string()));
    }
    
    let secret = jwt_secret();
    let token = generate_access_token(session.user_id, &secret)
        .map_err(|_| AppError::InternalServerError("Token generation failed".to_string()))?;
        
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": token
    })))
}

pub async fn logout(
    pool: web::Data<DbPool>,
    req_http: HttpRequest,
    req_body: web::Json<LogoutRequest>
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".to_string()))?;
    
    let _user_id = req_http.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Missing auth credentials".to_string()))?;

    let hashed_refresh = hash_token(&req_body.refresh_token);
    
    if let Ok(Some(session)) = Session::find_valid_by_token(&mut conn, &hashed_refresh) {
        let _ = Session::revoke_session(&mut conn, session.id);
    }
    
    Ok(HttpResponse::Ok().json("Logged out successfully"))
}
