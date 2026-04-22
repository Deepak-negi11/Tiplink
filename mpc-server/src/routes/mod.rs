pub mod dkg;
pub mod sign;
pub mod keys;

use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/dkg")
            .route("/round1", web::post().to(dkg::dkg_round1))
            .route("/round2", web::post().to(dkg::dkg_round2))
            .route("/finalize", web::post().to(dkg::dkg_finalize))
            .route("/store", web::post().to(dkg::store_share))
    );
    cfg.service(
        web::scope("/sign")
            .route("/init", web::post().to(sign::sign_init))
            .route("/round1", web::post().to(sign::sign_round1))
            .route("/round2", web::post().to(sign::sign_round2))
    );
    cfg.service(
        web::scope("/keys")
            .route("/pubkey_package", web::post().to(keys::get_pubkey_package))
    );
}
