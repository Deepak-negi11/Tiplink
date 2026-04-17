use actix_web::{post , web, HttpRequest , HttpResponse , Responder};
use serde::Deserialize;
use uuid::Uuid;

use crate::state::MpcState;
use crate::middleware;
use crate::vault;

#[derive(Deserialize)]
pub struct StoreShareRequest{
    pub user_id :Uuid,
    pub share_index:u16,
    pub secret_share:String
}

#[post("/shares/store")]
pub async fn store_sahre(
    req:HttpRequest,
    body_bytes: web::Bytes,
    server_state:web::Data<MpcState>
)-> impl Responder{

    if !middleware::is_authentic(&req , &body_bytes , &server_state.hmac_secret){
        return HttpResponse::Unauthorised().body("Hacker blocked");
    }

    let payload:StoreShareRequest = match serde_json::from_slice(&body_bytes){
        Ok(p) => p,
        Err(_) => return HttpResponse::BadRequest().body("Invalid Json payload"),
    
    };

    if payload.share_index != server_state.node_id{
        return HttpResponse::BadRequest().body("Wrong Node Id")
    }
    match vault::encrypt_and_save(payload.user_id, &payload.secret_share, &server_state.aes_secret_key) {
        Ok(_) => HttpResponse::Ok().body("Shard safely locked in the Vault."),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}