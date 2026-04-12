use actix_web::{web, HttpResponse};

pub async fn get_balance() -> HttpResponse {
    HttpResponse::Ok().json("Balance: Not Implemented")
}

pub async fn send() -> HttpResponse {
    HttpResponse::Ok().json("Send: Not Implemented")
}

pub async fn get_history() -> HttpResponse {
    HttpResponse::Ok().json("History: Not Implemented")
}
