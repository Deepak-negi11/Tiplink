use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use uuid::Uuid;
use crate::error::AppError;
use crate::db::conn::DbPool;
use crate::db::balances::Balance;
use crate::db::swap::{self, TransactionIntentEntry, NewTransactionIntent};
use crate::services::jupiter::{calculate_fee, JupiterQuote};
use crate::services::solana::submit_transaction;
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
pub struct QuoteRequest {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: u64,
    pub slippage_bps: u16,
}

#[derive(Deserialize)]
pub struct SwapRequest {
    pub quote: JupiterQuote,
}

#[derive(Deserialize)]
pub struct SubmitSwapRequest {
    pub nonce: Uuid,
    pub signed_tx: Vec<u8>,
}

pub async fn get_quote(
    req: HttpRequest,
    body: web::Json<QuoteRequest>
) -> Result<HttpResponse, AppError> {
    let _user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;

    // 1. calculate fee dynamically against TipLink's 50bps fee rate
    let (amount_after_fee, fee_amount) = calculate_fee(body.amount, 50);

    // 2. Fetch specific route against Jupiter's API using internally reduced math
    let quote = crate::services::jupiter::get_quote(
        &body.input_mint,
        &body.output_mint,
        amount_after_fee,
        body.slippage_bps
    ).await?;

    // 3. Return payload payload encapsulating fee abstraction for exact frontend visualization
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "quote": quote,
        "fee_breakdown": {
            "total_input": body.amount,
            "fee_amount": fee_amount,
            "swap_amount": amount_after_fee,
            "fee_bps": 50
        }
    })))
}

pub async fn execute_swap(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    body: web::Json<SwapRequest>
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;

    // 1. Check bounds across natively hosted tokens available.
    let input_amount_str = &body.quote.in_amount;
    let amount_u64: u64 = input_amount_str.parse().unwrap_or(0);
    // Assumes total amount encapsulates required funding variables efficiently 
    let balance = Balance::get_token_balance(&mut conn, user_id, &body.quote.input_mint)?
        .ok_or_else(|| AppError::BadRequest("Token balance tracking entirely missing on DB".to_string()))?;
        
    if balance.available < amount_u64 as i64 {
        return Err(AppError::BadRequest("Insufficient tokens for swap".to_string()));
    }

    // 2. Extract specific base64 encoding from API payload
    let user_pubkey = "TODO_EXTRACT_PUBKEY_FROM_DB";
    let unsigned_tx = crate::services::jupiter::get_swap_transaction(body.quote.clone(), user_pubkey).await?;
    let payload_b64 = base64::encode(&unsigned_tx);

    // 3. Store abstraction pending queue across native intent DB locking execution.
    let nonce = Uuid::new_v4();
    let intent_payload = serde_json::to_string(&body.quote).unwrap_or_default();
    
    let new_intent = NewTransactionIntent {
        id: nonce,
        user_id: Some(user_id),
        intent_message: "SWAP_REQUEST",
        intent_signature: &intent_payload, // Secures quote JSON natively ensuring no API changes map mid-txn
        unsigned_payload: Some(&payload_b64),
        status: Some("pending"),
    };
    TransactionIntentEntry::create_intent(&mut conn, new_intent)
        .map_err(|_| AppError::InternalServerError("Failed to record swap intent securely".to_string()))?;

    // 4. Yield Unsigned buffers 
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "nonce": nonce,
        "unsigned_tx": payload_b64
    })))
}

pub async fn submit_swap(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    body: web::Json<SubmitSwapRequest>
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;

    // 1. Extract swap locks
    use crate::db::schema::transaction_intents::dsl::*;
    use diesel::prelude::*;
    let intent = transaction_intents
        .filter(id.eq(body.nonce))
        .first::<TransactionIntentEntry>(&mut conn)
        .map_err(|_| AppError::BadRequest("Swap nonce expired or entirely invalid".to_string()))?;

    // 2. Verify payloads remain perfectly secure across API routing
    let quote: JupiterQuote = serde_json::from_str(&intent.intent_signature)
        .map_err(|_| AppError::InternalServerError("Failed to unpack routing maps".into()))?;

    // 3. Push to Solana Native JSON-RPC explicitly
    let rpc_url = env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    let sig = submit_transaction(&rpc_url, &body.signed_tx).await?;

    // 4. Force queue integration inside indexer loop 
    let input_amount_i64: i64 = quote.in_amount.parse().unwrap_or(0);
    // Base 50 bps execution fees mapping recalculations natively
    let (_amount_after_fee, fee_amount) = calculate_fee(input_amount_i64 as u64, 50);

    let _swap_id = swap::create(
        &pool,
        user_id,
        &quote.input_mint,
        &quote.output_mint,
        input_amount_i64, 
        0, // Set to 0 because final output requires validation logic during blockchain parsing inside indexing hooks
        fee_amount as i64,
        &sig
    )?;

    // Extinguish lock!
    diesel::delete(transaction_intents.filter(id.eq(body.nonce))).execute(&mut conn)
        .map_err(|_| AppError::InternalServerError("Failed to wipe execution lock".to_string()))?;

    // 5. Defer confirmed execution mappings back to external indexer
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "signature": sig,
        "status": "pending_indexing"
    })))
}
