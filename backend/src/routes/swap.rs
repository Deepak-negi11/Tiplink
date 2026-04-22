use actix_web::web;
use crate::handlers::swap_handler::{get_quote, execute_swap, submit_swap};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/swap")
            .route("/quote", web::post().to(get_quote))
            .route("/execute", web::post().to(execute_swap))
            .route("/submit", web::post().to(submit_swap))
    );
}