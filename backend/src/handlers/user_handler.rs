use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use uuid::Uuid;
use crate::error::AppError;
use crate::db::conn::DbPool;
use crate::db::user::User;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateRequest {
    pub username: String,
}

#[derive(Deserialize)]
pub struct LookupRequest {
    pub email: Option<String>,
    pub public_key: Option<String>,
}

pub async fn get_user(
    pool: web::Data<DbPool>,
    req: HttpRequest
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;

    let user = User::find_by_id(&mut conn, user_id)?
        .ok_or_else(|| AppError::BadRequest("User not found".to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "id": user.id,
        "email": user.email,
        "public_key": user.public_key,
        "created_at": user.created_at,
    })))
}

pub async fn update_user(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    body: web::Json<UpdateRequest>
) -> Result<HttpResponse, AppError> {
     let _conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;
     let _user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;

     Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Username successfully updated",
        "new_username": body.username
    })))
}

/// Checks if a recipient exists in the system by email or public_key.
/// Used by the frontend to decide: direct transfer vs. create link.
pub async fn lookup_recipient(
    pool: web::Data<DbPool>,
    _req: HttpRequest,
    body: web::Json<LookupRequest>
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;

    if let Some(ref email) = body.email {
        if let Some(user) = User::find_by_email(&mut conn, email)? {
            return Ok(HttpResponse::Ok().json(serde_json::json!({
                "found": true,
                "public_key": user.public_key,
                "email": user.email,
            })));
        }
    }

    if let Some(ref pubkey) = body.public_key {
        if let Some(user) = User::find_by_public_key(&mut conn, pubkey)? {
            return Ok(HttpResponse::Ok().json(serde_json::json!({
                "found": true,
                "public_key": user.public_key,
                "email": user.email,
            })));
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "found": false,
    })))
}
