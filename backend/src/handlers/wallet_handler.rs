use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use uuid::Uuid;
use crate::error::AppError;
use crate::db::conn::DbPool;
use crate::db::balances::Balance;
use crate::db::user::User;
use crate::db::transaction::Transaction;
use crate::services::intent::SendIntent;
use crate::services::solana::{build_transfer_tx, submit_transaction};
use crate::db::swap::{TransactionIntentEntry, NewTransactionIntent};
use serde::{Deserialize, Serialize};
use std::env;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

#[derive(Deserialize)]
pub struct SendRequest {
    pub to: String,
    pub amount: String,
    pub mint: String,
    pub timestamp: i64,
    pub signature: String,
}

#[derive(Deserialize)]
pub struct SubmitRequest {
    pub nonce: Uuid,
    pub signed_tx: Vec<u8>,
}

#[derive(Deserialize)]
pub struct HistoryParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Returns the required Solana RPC URL. Panics if not set — public RPC is rate-limited.
fn solana_rpc_url() -> String {
    env::var("SOLANA_RPC_URL")
        .expect("FATAL: SOLANA_RPC_URL must be set. The public mainnet RPC is rate-limited and unsuitable for production.")
}

/// Extracts the authenticated user's public key from the database.
fn get_user_pubkey(conn: &mut diesel::PgConnection, user_id: Uuid) -> Result<String, AppError> {
    let user = User::find_by_id(conn, user_id)?
        .ok_or_else(|| AppError::InternalServerError("Authenticated user not found in database".to_string()))?;
    Ok(user.public_key)
}

pub async fn get_balance(
    pool: web::Data<DbPool>,
    req: HttpRequest
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;

    let balances = Balance::get_user_balances(&mut conn, user_id)?;
    Ok(HttpResponse::Ok().json(balances))
}

pub async fn send(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    body: web::Json<SendRequest>
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;
        
    let user_pubkey = get_user_pubkey(&mut conn, user_id)?;
    
    let _intent = SendIntent {
        to: body.to.clone(),
        amount: body.amount.clone(),
        mint: body.mint.clone(),
        timestamp: body.timestamp,
        signature: body.signature.clone(),
    };
    
    let amount_u64: u64 = body.amount.parse().map_err(|_| AppError::BadRequest("Invalid amount format".to_string()))?;
    let balance = Balance::get_token_balance(&mut conn, user_id, &body.mint)?
        .ok_or_else(|| AppError::BadRequest("Token balance not found".to_string()))?;
        
    if balance.available < amount_u64 as i64 {
        return Err(AppError::BadRequest("Insufficient balance for requested transfer".to_string()));
    }
    
    let rpc_url = solana_rpc_url();
    let rpc_client = solana_client::nonblocking::rpc_client::RpcClient::new(rpc_url);
    let unsigned_tx = build_transfer_tx(&rpc_client, &user_pubkey, &body.to, amount_u64).await?;
    let payload_b64 = BASE64.encode(&unsigned_tx);
    
    let nonce = Uuid::new_v4();
    let intent_meta = format!("SEND|{}|{}|{}", body.to, body.amount, body.mint);
    let new_intent = NewTransactionIntent {
        id: nonce,
        user_id: Some(user_id),
        intent_message: &intent_meta,
        intent_signature: &body.signature,
        unsigned_payload: Some(&payload_b64),
        status: Some("pending"),
    };
    TransactionIntentEntry::create_intent(&mut conn, new_intent)
        .map_err(|_| AppError::InternalServerError("Failed to map transaction intent".to_string()))?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "nonce": nonce,
        "unsigned_tx": payload_b64
    })))
}

pub async fn submit_send(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    body: web::Json<SubmitRequest>
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;
    let user_uuid = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;

    use crate::db::schema::transaction_intents::dsl::*;
    use diesel::prelude::*;
    let pending = transaction_intents
        .filter(id.eq(body.nonce))
        .first::<TransactionIntentEntry>(&mut conn)
        .map_err(|_| AppError::BadRequest("Invalid or expired pending_tx nonce".to_string()))?;
    
    let intent_parts: Vec<&str> = pending.intent_message.split('|').collect();
    let (original_mint, original_amount) = if intent_parts.len() >= 4 {
        (intent_parts[3].to_string(), intent_parts[2].parse::<i64>().unwrap_or(0))
    } else {
        return Err(AppError::InternalServerError("Corrupt intent message format".into()));
    };
        
    let rpc_url = solana_rpc_url();
    let rpc_client = solana_client::nonblocking::rpc_client::RpcClient::new(rpc_url);
    let sig = submit_transaction(&rpc_client, &String::from_utf8(body.signed_tx.clone()).unwrap_or_default()).await?;
    
    let _ = Balance::subtract_balance(&mut conn, user_uuid, &original_mint, original_amount);
    
    diesel::delete(transaction_intents.filter(id.eq(body.nonce))).execute(&mut conn)
        .map_err(|_| AppError::InternalServerError("Failed to clear intent lock cleanly".to_string()))?;
        
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "signature": sig
    })))
}

pub async fn get_history(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    query: web::Query<HistoryParams>
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;
        
    let limit = query.limit.unwrap_or(20).min(100); // Cap at 100
    let offset = query.offset.unwrap_or(0);
    
    use crate::db::schema::transactions::dsl;
    use diesel::prelude::*;
    
    let txs: Vec<Transaction> = dsl::transactions
        .filter(dsl::user_id.eq(user_id))
        .order(dsl::block_time.desc())
        .limit(limit)
        .offset(offset)
        .select(Transaction::as_select())
        .load(&mut conn)
        .map_err(|e| AppError::InternalServerError(format!("Failed to query transaction history: {}", e)))?;
    
    Ok(HttpResponse::Ok().json(txs))
}
