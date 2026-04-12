use actix_web::{web, HttpResponse, HttpMessage};
use crate::db::conn::DbPool;
use crate::error::AppError;
use uuid::Uuid;

pub async fn create_link(
    pool: web::Data<DbPool>,
    req: actix_web::HttpRequest
) -> Result<HttpResponse, AppError> {
    let _user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;
        
    Ok(HttpResponse::Ok().json("Create Link: Connected to Auth guard, waiting for DB integration"))
}

pub async fn get_link(
    pool: web::Data<DbPool>,
    path: web::Path<String>
) -> Result<HttpResponse, AppError> {
    let _id = path.into_inner();
    Ok(HttpResponse::Ok().json("Get Link: Ready for DB querying"))
}

pub async fn claim_link(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    req: actix_web::HttpRequest
) -> Result<HttpResponse, AppError> {
    let _user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;
    let _id = path.into_inner();

    Ok(HttpResponse::Ok().json("Claim Link: Reassigned wallet ownership successfully (Stub)"))
}
