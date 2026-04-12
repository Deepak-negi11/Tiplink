use actix_web::web;
use crate::handlers::link_handler::{create_link, get_link, claim_link};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/link")
            .route("/create", web::post().to(create_link))
            .route("/{id}", web::get().to(get_link))
            .route("/{id}/claim", web::post().to(claim_link))
    );
}