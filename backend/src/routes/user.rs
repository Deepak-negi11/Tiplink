use actix_web::web;
use crate::handlers::user_handler::{get_user, update_user};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .route("", web::get().to(get_user))
            .route("", web::patch().to(update_user))
    );
}