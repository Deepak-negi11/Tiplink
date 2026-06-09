use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use uuid::Uuid;
use crate::error::AppError;
use crate::db::conn::DbPool;
use crate::db::balances::Balance;
use crate::db::user::User;
use crate::db::transaction::{NewTransaction, Transaction, TxType};
use crate::services::mpc::{coordinate_transaction_signature, mpc_config};
use crate::services::solana::{build_transfer_tx, submit_transaction, to_db_mint, to_client_mint};
use serde::Deserialize;
use std::env;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use solana_sdk::{pubkey::Pubkey, signature::Signature, transaction::Transaction as SolanaTransaction};
use std::str::FromStr;
use chrono::Utc;
use std::collections::HashMap;

const TOKEN_PROGRAMS: [&str; 2] = [
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
];

fn token_symbol(mint: &str) -> &str {
    match mint {
        "So11111111111111111111111111111111111111112" => "SOL",
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "USDC",
        "Es9vMFrzaCERmJfrF4H2FYDapipNi2aBcVBPjL6dLrH" => "USDT",
        "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh" => "WBTC",
        _ => "TOKEN",
    }
}

async fn get_spl_token_balances(
    rpc_url: &str,
    owner: &str,
) -> Result<HashMap<String, (i64, i16)>, reqwest::Error> {
    let client = reqwest::Client::new();
    let mut balances = HashMap::new();

    for program_id in TOKEN_PROGRAMS {
        let response: serde_json::Value = client
            .post(rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getTokenAccountsByOwner",
                "params": [
                    owner,
                    { "programId": program_id },
                    { "encoding": "jsonParsed" }
                ]
            }))
            .send()
            .await?
            .json()
            .await?;

        if let Some(accounts) = response.pointer("/result/value").and_then(|value| value.as_array()) {
            for account in accounts {
                let info = account.pointer("/account/data/parsed/info");
                let mint = info.and_then(|value| value.get("mint")).and_then(|value| value.as_str());
                let amount = info
                    .and_then(|value| value.pointer("/tokenAmount/amount"))
                    .and_then(|value| value.as_str())
                    .and_then(|value| value.parse::<i64>().ok());
                let decimals = info
                    .and_then(|value| value.pointer("/tokenAmount/decimals"))
                    .and_then(|value| value.as_i64())
                    .and_then(|value| i16::try_from(value).ok());

                if let (Some(mint), Some(amount), Some(decimals)) = (mint, amount, decimals) {
                    let entry = balances.entry(mint.to_string()).or_insert((0_i64, decimals));
                    entry.0 = entry.0.saturating_add(amount);
                }
            }
        }
    }

    Ok(balances)
}

async fn get_native_sol_balance(rpc_url: &str, owner: &str) -> Result<Option<i64>, reqwest::Error> {
    let response: serde_json::Value = reqwest::Client::new()
        .post(rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBalance",
            "params": [owner]
        }))
        .send()
        .await?
        .json()
        .await?;

    Ok(response.pointer("/result/value").and_then(|value| value.as_i64()))
}

#[derive(Deserialize)]
pub struct SendRequest {
    pub to: String,
    pub amount: f64,
    pub mint: String,
}

#[derive(Deserialize)]
pub struct HistoryParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Returns the required Solana RPC URL based on header.
fn solana_rpc_url(req: &HttpRequest) -> String {
    if let Some(net) = req.headers().get("x-network") {
        if net.to_str().unwrap_or("").to_lowercase() == "devnet" {
            return "https://api.devnet.solana.com".to_string();
        }
    }
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

    let is_devnet = if let Some(net) = req.headers().get("x-network") {
        net.to_str().unwrap_or("").to_lowercase() == "devnet"
    } else {
        false
    };

    let user_pubkey = get_user_pubkey(&mut conn, user_id)?;

    // Sync on-chain SOL balance for the active network
    let rpc_url = solana_rpc_url(&req);
    if Pubkey::from_str(&user_pubkey).is_ok() {
        if let Ok(Some(sol_balance)) = get_native_sol_balance(&rpc_url, &user_pubkey).await {
            let db_sol_mint = to_db_mint("So11111111111111111111111111111111111111112", is_devnet);
            Balance::sync_onchain_balance(&mut conn, user_id, &db_sol_mint, "SOL", sol_balance, 9)?;
        }
        if let Ok(token_balances) = get_spl_token_balances(&rpc_url, &user_pubkey).await {
            for (mint, (amount, decimals)) in token_balances {
                let db_mint = to_db_mint(&mint, is_devnet);
                Balance::sync_onchain_balance(
                    &mut conn,
                    user_id,
                    &db_mint,
                    token_symbol(&mint),
                    amount,
                    decimals,
                )?;
            }
        }
    }

    let db_balances = Balance::get_user_balances(&mut conn, user_id)?;
    let filtered_balances: Vec<serde_json::Value> = db_balances
        .into_iter()
        .filter(|b| {
            if is_devnet {
                b.token_mint.starts_with("devnet_")
            } else {
                !b.token_mint.starts_with("devnet_")
            }
        })
        .map(|b| {
            serde_json::json!({
                "id": b.id,
                "mint": to_client_mint(&b.token_mint),
                "symbol": b.token_symbol,
                "amount": b.amount,
                "available": b.available,
                "locked": b.locked,
                "decimals": b.decimals,
                "updated_at": b.updated_at,
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(filtered_balances))
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

    let is_devnet = if let Some(net) = req.headers().get("x-network") {
        net.to_str().unwrap_or("").to_lowercase() == "devnet"
    } else {
        false
    };
    let db_mint = to_db_mint(&body.mint, is_devnet);

    if body.mint != "So11111111111111111111111111111111111111112" {
        return Err(AppError::BadRequest(
            "Direct transfers currently support SOL only.".to_string(),
        ));
    }

    if !body.amount.is_finite() || body.amount <= 0.0 {
        return Err(AppError::BadRequest("Enter a valid SOL amount.".to_string()));
    }

    let lamports = (body.amount * 1_000_000_000.0).round();
    if lamports < 1.0 {
        return Err(AppError::BadRequest("Amount is below one lamport.".to_string()));
    }
    if lamports > i64::MAX as f64 {
        return Err(AppError::BadRequest("Amount is too large.".to_string()));
    }
    let amount_u64 = lamports as u64;

    Pubkey::from_str(&body.to)
        .map_err(|_| AppError::BadRequest("Enter a valid Solana wallet address.".to_string()))?;

    let balance = Balance::get_token_balance(&mut conn, user_id, &db_mint)?
        .ok_or_else(|| AppError::BadRequest("Token balance not found".to_string()))?;
        
    if balance.available < amount_u64 as i64 {
        return Err(AppError::BadRequest("Insufficient balance for requested transfer".to_string()));
    }
    
    let rpc_url = solana_rpc_url(&req);
    let rpc_client = solana_client::nonblocking::rpc_client::RpcClient::new(rpc_url);
    let unsigned_tx_b64 = build_transfer_tx(&rpc_client, &user_pubkey, &body.to, amount_u64).await?;
    let unsigned_tx_bytes = BASE64.decode(&unsigned_tx_b64)
        .map_err(|_| AppError::InternalServerError("Failed to decode unsigned transaction.".to_string()))?;
    let mut transaction: SolanaTransaction = bincode::deserialize(&unsigned_tx_bytes)
        .map_err(|_| AppError::InternalServerError("Failed to parse unsigned transaction.".to_string()))?;

    let signature_bytes = coordinate_transaction_signature(
        &mpc_config()?,
        user_id,
        &transaction.message_data(),
    ).await?;
    let signature = Signature::try_from(signature_bytes.as_slice())
        .map_err(|_| AppError::InternalServerError("MPC returned an invalid Solana signature.".to_string()))?;

    if transaction.signatures.is_empty() {
        return Err(AppError::InternalServerError("Transaction has no signer slot.".to_string()));
    }
    transaction.signatures[0] = signature;

    let signed_tx_b64 = BASE64.encode(
        bincode::serialize(&transaction)
            .map_err(|_| AppError::InternalServerError("Failed to serialize signed transaction.".to_string()))?
    );
    let tx_hash = submit_transaction(&rpc_client, &signed_tx_b64).await?;

    Balance::subtract_balance(&mut conn, user_id, &db_mint, amount_u64 as i64)?;
    let new_tx = NewTransaction {
        user_id,
        amount: amount_u64 as i64,
        token_mint: &db_mint,
        token_symbol: "SOL",
        tx_hash: &tx_hash,
        tx_type: TxType::Transfer,
        from_address: &user_pubkey,
        to_address: &body.to,
        slot: 0,
        block_time: Utc::now(),
    };
    Transaction::insert_tx(&mut conn, new_tx)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "signature": tx_hash
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
    
    let is_devnet = if let Some(net) = req.headers().get("x-network") {
        net.to_str().unwrap_or("").to_lowercase() == "devnet"
    } else {
        false
    };

    let mut query_builder = dsl::transactions
        .filter(dsl::user_id.eq(user_id))
        .into_boxed();

    if is_devnet {
        query_builder = query_builder.filter(dsl::token_mint.like("devnet_%"));
    } else {
        query_builder = query_builder.filter(dsl::token_mint.not_like("devnet_%"));
    }

    let txs: Vec<Transaction> = query_builder
        .order(dsl::block_time.desc())
        .limit(limit)
        .offset(offset)
        .select(Transaction::as_select())
        .load(&mut conn)
        .map_err(|e| AppError::InternalServerError(format!("Failed to query transaction history: {}", e)))?;
    
    let mapped_txs: Vec<Transaction> = txs
        .into_iter()
        .map(|mut tx| {
            tx.token_mint = to_client_mint(&tx.token_mint);
            tx
        })
        .collect();

    Ok(HttpResponse::Ok().json(mapped_txs))
}
