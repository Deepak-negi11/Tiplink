use actix_web::{web, HttpResponse};

pub async fn get_quote() -> HttpResponse {
    HttpResponse::Ok().json("Quote: Not Implemented")
}

pub async fn execute_swap() -> HttpResponse {
    HttpResponse::Ok().json("Execute: Not Implemented")
}

pub async fn submit_tx() -> HttpResponse {
    HttpResponse::Ok().json("Submit: Not Implemented")
}
