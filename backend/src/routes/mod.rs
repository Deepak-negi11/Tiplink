pub mod auth;
pub mod link;
pub mod swap;
pub mod user;
pub mod wallet;

use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(auth::configure)
            .configure(link::configure)
            .configure(swap::configure)
            .configure(user::configure)
            .configure(wallet::configure)
    );
}