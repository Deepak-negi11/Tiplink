use actix_web::{web, HttpResponse};

pub async fn get_user() -> HttpResponse {
    HttpResponse::Ok().json("Get User: Not Implemented")
}

pub async fn update_user() -> HttpResponse {
    HttpResponse::Ok().json("Update User: Not Implemented")
}
