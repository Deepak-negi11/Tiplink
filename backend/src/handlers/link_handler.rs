use actix_web::{web, HttpResponse};

pub async fn create_link() -> HttpResponse {
    HttpResponse::Ok().json("Create Link: Not Implemented")
}

pub async fn get_link() -> HttpResponse {
    HttpResponse::Ok().json("Get Link: Not Implemented")
}

pub async fn claim_link() -> HttpResponse {
    HttpResponse::Ok().json("Claim Link: Not Implemented")
}
