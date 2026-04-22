use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use uuid::Uuid;
use crate::error::AppError;
use crate::db::conn::DbPool;
use crate::db::balances::Balance;
use crate::db::user::User;
use crate::db::swap::{self, TransactionIntentEntry, NewTransactionIntent};
use crate::services::jupiter::{calculate_fee, JupiterQuote};
use crate::services::solana::submit_transaction;
use serde::Deserialize;
use std::env;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

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

/// Returns the required Solana RPC URL. Panics if not set.
fn solana_rpc_url() -> String {
    env::var("SOLANA_RPC_URL")
        .expect("FATAL: SOLANA_RPC_URL must be set.")
}

/// Extracts the authenticated user's public key from the database.
fn get_user_pubkey(conn: &mut diesel::PgConnection, user_id: Uuid) -> Result<String, AppError> {
    let user = User::find_by_id(conn, user_id)?
        .ok_or_else(|| AppError::InternalServerError("Authenticated user not found in database".to_string()))?;
    Ok(user.public_key)
}

pub async fn get_quote(
    req: HttpRequest,
    body: web::Json<QuoteRequest>
) -> Result<HttpResponse, AppError> {
    let _user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;

    // 1. calculate fee dynamically against TipLink's 50bps fee rate
    let (amount_after_fee, fee_amount) = calculate_fee(body.amount, 50);

    // 2. Fetch specific route against Jupiter's API
    let quote = crate::services::jupiter::get_quote(
        &body.input_mint,
        &body.output_mint,
        amount_after_fee,
        body.slippage_bps
    ).await?;

    // 3. Return payload encapsulating fee abstraction for exact frontend visualization
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

    // 1. Check balance
    let input_amount_str = &body.quote.in_amount;
    let amount_u64: u64 = input_amount_str.parse().unwrap_or(0);
    let balance = Balance::get_token_balance(&mut conn, user_id, &body.quote.input_mint)?
        .ok_or_else(|| AppError::BadRequest("Token balance tracking entirely missing on DB".to_string()))?;
        
    if balance.available < amount_u64 as i64 {
        return Err(AppError::BadRequest("Insufficient tokens for swap".to_string()));
    }

    // 2. Extract real user pubkey from DB for Jupiter swap transaction
    let user_pubkey = get_user_pubkey(&mut conn, user_id)?;
    let unsigned_tx = crate::services::jupiter::get_swap_transaction(body.quote.clone(), &user_pubkey).await?;
    let payload_b64 = BASE64.encode(&unsigned_tx);

    // 3. Store intent in DB
    let nonce = Uuid::new_v4();
    let intent_payload = serde_json::to_string(&body.quote).unwrap_or_default();
    
    let new_intent = NewTransactionIntent {
        id: nonce,
        user_id: Some(user_id),
        intent_message: "SWAP_REQUEST",
        intent_signature: &intent_payload,
        unsigned_payload: Some(&payload_b64),
        status: Some("pending"),
    };
    TransactionIntentEntry::create_intent(&mut conn, new_intent)
        .map_err(|_| AppError::InternalServerError("Failed to record swap intent securely".to_string()))?;

    // 4. Return unsigned transaction
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
    let user_uuid = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;

    // 1. Retrieve and validate the intent
    use crate::db::schema::transaction_intents::dsl::*;
    use diesel::prelude::*;
    let intent = transaction_intents
        .filter(id.eq(body.nonce))
        .first::<TransactionIntentEntry>(&mut conn)
        .map_err(|_| AppError::BadRequest("Swap nonce expired or entirely invalid".to_string()))?;

    // 2. Deserialize the stored quote
    let quote: JupiterQuote = serde_json::from_str(&intent.intent_signature)
        .map_err(|_| AppError::InternalServerError("Failed to unpack routing maps".into()))?;

    // 3. Submit to Solana
    let rpc_url = solana_rpc_url();
    let rpc_client = solana_client::nonblocking::rpc_client::RpcClient::new(rpc_url);
    let sig = submit_transaction(&rpc_client, &String::from_utf8(body.signed_tx.clone()).unwrap_or_default()).await?;

    // 4. Record swap in history with real amounts
    let input_amount_i64: i64 = quote.in_amount.parse().unwrap_or(0);
    let (_amount_after_fee, fee_amount) = calculate_fee(input_amount_i64 as u64, 50);

    let new_swap = crate::db::swap::NewSwapEntry {
        user_id: user_uuid,
        input_mint: &quote.input_mint,
        output_mint: &quote.output_mint,
        output_amount: 0, // Updated by indexer once confirmed
        input_amount: input_amount_i64,
        fee_amount: fee_amount as i64,
        price_impact: quote.price_impact_pct.parse().unwrap_or_default(),
        tx_hash: &sig,
        status: swap::SwapStatus::Pending,
        requested_slippage_bps: 50,
    };
    let _swap_id = crate::db::swap::SwapEntry::create_entry(&mut conn, new_swap)?;

    // 5. Subtract input balance immediately (credit handled by indexer on confirmation)
    let _ = Balance::subtract_balance(&mut conn, user_uuid, &quote.input_mint, input_amount_i64);

    // 6. Clean up intent lock
    diesel::delete(transaction_intents.filter(id.eq(body.nonce))).execute(&mut conn)
        .map_err(|_| AppError::InternalServerError("Failed to wipe execution lock".to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "signature": sig,
        "status": "pending_indexing"
    })))
}
