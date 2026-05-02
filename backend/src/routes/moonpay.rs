use actix_web::web;
use crate::handlers::moonpay_handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/moonpay")
            .route("/sign-url", web::post().to(moonpay_handlers::sign_url))
            .route("/webhook",  web::post().to(moonpay_handlers::webhook))
            .route("/limits",   web::get().to(moonpay_handlers::get_limits))
            .route("/quote",    web::get().to(moonpay_handlers::get_quote))
    );
}
