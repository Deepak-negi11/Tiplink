use actix_web::{get, App,HttpServer ,web, HttpResponse,Responder};

use dotevny::dotenv;
use std::env;

use crate::state::MpcState;

mod config;
mod error;
mod state;
mod routes;
mod crypto;

#[get("/ping")]
async fn ping() -> impl Responder {
    HttpResponse::Ok().body("pong - The MPC Vault Manager is awake!")
}

#[actix_web::main]
async fn main()->std::io::Result<()>{
    println!("Starting the MPC Vault Server on port 8081:");
    dotenv.ok();

    let node_id_str = env::var("NODE_ID").expect("Node_id must be set");
    let hmac_key = env::var("INTERNAL_MPC_KEY").expect("Internal_mpc_key must eb set");
    let aes_key = env::var("AES_MASTER_KEY").expect("AES MASTER KEY must be set");

    let app_state = MpcState{
        node_id:node_id_str.parse::<u16>().expect("Node Id must be a number "),
        hmac_secret:hmac_key,
        aes_secret_key:aes_key,
    };

    HttpServer::new(|| {
        App::new()
            .server()
    })
    .bind(("127.0.0.1",8081))?
    .run()
    .await()
}