use actix_web::web;
use crate::handlers::wallet_handler::{get_balance, send, get_history};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/wallet")
            .route("/balance", web::get().to(get_balance))
            .route("/send", web::post().to(send))
            .route("/history", web::get().to(get_history))
    );
}