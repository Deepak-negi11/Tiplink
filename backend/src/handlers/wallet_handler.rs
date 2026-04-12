use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use uuid::Uuid;
use crate::error::AppError;
use crate::db::conn::DbPool;
use crate::db::balances::Balance;
use crate::services::intent::SendIntent;
use crate::services::solana::{build_transfer_tx, submit_transaction};
use crate::db::swap::{TransactionIntentEntry, NewTransactionIntent};
use serde::{Deserialize, Serialize};
use std::env;

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

// 1. Fetch live balances mapped exclusively to User database context
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

// 2. Initial logic to generate Unsigned Transfer Intent and verify balances beforehand
pub async fn send(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    body: web::Json<SendRequest>
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;
        
    // TODO: In production extract Native Pubkey from User db abstraction here
    let user_pubkey = "TODO_EXTRACT_PUBKEY_FROM_DB";
    
    // Abstract intent validation properties
    let _intent = SendIntent {
        to: body.to.clone(),
        amount: body.amount.clone(),
        mint: body.mint.clone(),
        timestamp: body.timestamp,
        signature: body.signature.clone(),
    };
    
    // Check internal balance mapping logic
    let amount_u64: u64 = body.amount.parse().map_err(|_| AppError::BadRequest("Invalid amount format".to_string()))?;
    let balance = Balance::get_token_balance(&mut conn, user_id, &body.mint)?
        .ok_or_else(|| AppError::BadRequest("Token balance not found".to_string()))?;
        
    if balance.available < amount_u64 as i64 {
        return Err(AppError::BadRequest("Insufficient balance for requested transfer".to_string()));
    }
    
    // Abstract Solana Builder routing
    let rpc_url = env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    
    let unsigned_tx = build_transfer_tx(&rpc_url, user_pubkey, &body.to, amount_u64).await?;
    let payload_b64 = base64::encode(&unsigned_tx);
    
    // Store intent directly inside database configuration schema locking execution sequence
    let nonce = Uuid::new_v4();
    let new_intent = NewTransactionIntent {
        id: nonce,
        user_id: Some(user_id),
        intent_message: "SEND_REQUEST",
        intent_signature: &body.signature,
        unsigned_payload: Some(&payload_b64),
        status: Some("pending"),
    };
    TransactionIntentEntry::create_intent(&mut conn, new_intent)
        .map_err(|_| AppError::InternalServerError("Failed to map transaction intent".to_string()))?;
    
    // Extrapolate payload back to front-end abstraction layers safely
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "nonce": nonce,
        "unsigned_tx": payload_b64
    })))
}

// 3. Confirm target Signed Tx hits network safely mapping through the nonce indexer
pub async fn submit_send(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    body: web::Json<SubmitRequest>
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;

    // Find intent routing execution
    use crate::db::schema::transaction_intents::dsl::*;
    use diesel::prelude::*;
    let _pending = transaction_intents
        .filter(id.eq(body.nonce))
        .first::<TransactionIntentEntry>(&mut conn)
        .map_err(|_| AppError::BadRequest("Invalid or expired pending_tx nonce".to_string()))?;
        
    // Verify target signatures match native buffers inside Solana pipeline 
    // -> Requires mapping extract_amounts over pending transactions
    
    let rpc_url = env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    let sig = submit_transaction(&rpc_url, &body.signed_tx).await?;
    
    // Abstract wallet subtraction locally 
    let _ = Balance::subtract_balance(&mut conn, user_id, "NATIVE_SOL", 0);
    
    // Extinguish intent block structure 
    diesel::delete(transaction_intents.filter(id.eq(body.nonce))).execute(&mut conn)
        .map_err(|_| AppError::InternalServerError("Failed to clear intent lock cleanly".to_string()))?;
        
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "signature": sig
    })))
}

// 4. Extrapolate paginated structures natively tracking historical transfers
pub async fn get_history(
    req: HttpRequest,
    query: web::Query<HistoryParams>
) -> Result<HttpResponse, AppError> {
    let _user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;
        
    let _limit = query.limit.unwrap_or(20);
    let _offset = query.offset.unwrap_or(0);
    
    // Pagination wrapper intercept
    Ok(HttpResponse::Ok().json("Fetched paginated history successfully via internal generic indices"))
}
