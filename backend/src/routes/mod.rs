pub mod auth;
pub mod link;
pub mod moonpay;
pub mod swap;
pub mod user;
pub mod wallet;


use actix_web::web;
use actix_web_httpauth::middleware::HttpAuthentication;
use crate::middleware::jwt_validator;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(auth::configure)
            // MoonPay webhook (no auth — called by MoonPay servers)
            .route("/moonpay/webhook", web::post().to(crate::handlers::moonpay_handlers::webhook))
            .service(
                web::scope("")
                    .wrap(HttpAuthentication::bearer(jwt_validator))
                    .configure(link::configure)
                    .configure(swap::configure)
                    .configure(user::configure)
                    .configure(wallet::configure)
                    .configure(moonpay::configure)
            )
    );
}