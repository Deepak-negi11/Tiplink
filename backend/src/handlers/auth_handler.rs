use actix_web::{web, HttpResponse};
use crate::models::auth::{SignupRequest, SigninRequest};
use crate::db::conn::DbPool;
use crate::error::AppError;

pub async fn signup(
    pool: web::Data<DbPool>,
    req: web::Json<SignupRequest>
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json("Signup: Not Implemented"))
}

pub async fn signin(
    pool: web::Data<DbPool>,
    req: web::Json<SigninRequest>
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json("Signin: Not Implemented"))
}

pub async fn refresh() -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json("Refresh: Not Implemented"))
}
