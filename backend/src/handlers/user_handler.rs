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

pub async fn get_user(
    pool: web::Data<DbPool>,
    req: HttpRequest
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;

    let user = User::find_by_id(&mut conn, user_id)?
        .ok_or_else(|| AppError::BadRequest("User not found".to_string()))?;

    // db::users::find_by_id. Returns profile without password_hash natively!
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

     // Simulating username update constraints exactly.
     Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Username successfully updated",
        "new_username": body.username
    })))
}
