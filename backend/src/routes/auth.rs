use actix_web::web;
use crate::handlers::auth_handler::{signup, signin, refresh};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/signup", web::post().to(signup))
            .route("/signin", web::post().to(signin))
            .route("/refresh", web::post().to(refresh))
    );
}