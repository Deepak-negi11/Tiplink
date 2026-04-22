use actix_web::{get, App, HttpServer, web, HttpResponse, Responder};
use dotenvy::dotenv;
use std::env;

use crate::state::MpcState;

mod config;
mod error;
mod state;
mod vault;
mod routes;
mod crypto;
mod middleware;
mod util;

#[get("/ping")]
async fn ping() -> impl Responder {
    HttpResponse::Ok().body("pong - The MPC Vault Manager is awake!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let node_id_str = env::var("NODE_ID").expect("NODE_ID must be set");
    let hmac_key = env::var("INTERNAL_MPC_KEY").expect("INTERNAL_MPC_KEY must be set");
    let aes_key = env::var("AES_MASTER_KEY").expect("AES_MASTER_KEY must be set");

    let node_id: u16 = node_id_str.parse().expect("NODE_ID must be a number");
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8081".to_string())
        .parse()
        .expect("PORT must be a number");

    println!("Starting MPC Vault Server (Node {}) on port {}...", node_id, port);

    let app_state = MpcState::new(node_id, hmac_key, aes_key);

    let cleanup_state = app_state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            cleanup_state.purge_expired_sessions();
        }
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(ping)
            .configure(routes::init_routes)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}