use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use uuid::Uuid;
use chrono::{Utc, TimeDelta};
use crate::error::AppError;
use crate::db::conn::DbPool;
use crate::db::balances::Balance;
use crate::db::user::User;
use crate::db::links::{PaymentLink, NewPaymentLink, LinkStatus};
use crate::services::solana::{build_transfer_tx, submit_transaction};
use solana_client::nonblocking::rpc_client::RpcClient;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use crate::services::mpc::coordinate_transaction_signature;
use crate::services::dkg::Config;
use crate::models::link::{CreateLinkRequest, ClaimRequest};
use sha2::{Sha256, Digest};
use rand::{distributions::Alphanumeric, Rng};
use std::env;

/// Returns the required Solana RPC URL.
fn solana_rpc_url() -> String {
    env::var("SOLANA_RPC_URL")
        .expect("FATAL: SOLANA_RPC_URL must be set.")
}

/// Builds MPC node configuration from environment variables.
fn mpc_config() -> Result<Config, AppError> {
    let aws = env::var("MPC_NODE_1")
        .map_err(|_| AppError::InternalServerError("MPC_NODE_1 not configured".into()))?;
    let do_ocean = env::var("MPC_NODE_2")
        .map_err(|_| AppError::InternalServerError("MPC_NODE_2 not configured".into()))?;
    let cloudflare = env::var("MPC_NODE_3")
        .map_err(|_| AppError::InternalServerError("MPC_NODE_3 not configured".into()))?;
    let api_keys = env::var("INTERNAL_MPC_KEY").unwrap_or_default();
    Ok(Config { aws, do_ocean, cloudflare, api_keys })
}

/// Extracts the authenticated user's public key from the database.
fn get_user_pubkey(conn: &mut diesel::PgConnection, user_id: Uuid) -> Result<String, AppError> {
    let user = User::find_by_id(conn, user_id)?
        .ok_or_else(|| AppError::InternalServerError("User not found".to_string()))?;
    Ok(user.public_key)
}

/// Look up the decimal precision for a given token mint from the user's balance table.
fn get_token_decimals(conn: &mut diesel::PgConnection, user_id: Uuid, mint: &str) -> Result<i16, AppError> {
    let balance = Balance::get_token_balance(conn, user_id, mint)?;
    match balance {
        Some(b) => Ok(b.decimals),
        None => {
            // Default decimals by well-known mint addresses
            match mint {
                // USDC
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => Ok(6),
                // USDT
                "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => Ok(6),
                // Native SOL (wrapped SOL mint)
                "So11111111111111111111111111111111111111112" => Ok(9),
                _ => Err(AppError::BadRequest(format!("Unknown token mint: {}. Cannot determine decimals.", mint))),
            }
        }
    }
}

pub async fn create_link(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    body: web::Json<CreateLinkRequest>
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;

    // Look up correct decimals for this token mint
    let decimals = get_token_decimals(&mut conn, user_id, &body.token_mint)?;
    let multiplier = 10_i64.pow(decimals as u32);
    let amount_i64 = (body.amount * multiplier as f64) as i64;

    let balance = Balance::get_token_balance(&mut conn, user_id, &body.token_mint)?
        .ok_or_else(|| AppError::BadRequest("Token balance not found".to_string()))?;
        
    if balance.available < amount_i64 {
        return Err(AppError::BadRequest("Insufficient available balance".to_string()));
    }

    // 2. generate secret_code=random_string()
    let secret_code: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    // 3. claim_hash=SHA256(secret_code)
    let mut hasher = Sha256::new();
    hasher.update(secret_code.as_bytes());
    let claim_hash = format!("{:x}", hasher.finalize());

    // 4. lock balance locally preventing double-spends
    Balance::lock_funds(&mut conn, user_id, &body.token_mint, amount_i64)?;

    // 5. Derive a deterministic escrow PDA from the user's pubkey + link seed
    let user_pubkey = get_user_pubkey(&mut conn, user_id)?;
    let link_id = Uuid::new_v4();
    let seed = format!("tiplink_escrow_{}", link_id);
    let escrow_pda = derive_escrow_pda(&user_pubkey, &seed)?;

    // 6. db::links::create
    let new_link = NewPaymentLink {
        id: link_id,
        creator_id: user_id,
        escrow_pda: &escrow_pda,
        claim_hash: &claim_hash,
        token_mint: &body.token_mint,
        amount: amount_i64,
        recipient_email: None,
        recipient_phone: None,
        status: LinkStatus::Active,
        expires_at: Utc::now() + TimeDelta::try_days(3).unwrap_or_default(),
        memo: None,
    };
    PaymentLink::create_link(&mut conn, new_link)
        .map_err(|_| AppError::InternalServerError("Failed to create link record".to_string()))?;

    // 7. Return link_id and the claim URL with the secret embedded.
    // The secret is returned ONLY to the creator so they can share it.
    // It is NOT stored server-side (only the hash is stored).
    let claim_url = format!("/claim/{}?code={}", link_id, secret_code);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "link_id": link_id.to_string(),
        "claim_url": claim_url,
        "status": "locked_and_active"
    })))
}

/// Derive an escrow PDA (Program Derived Address) deterministically.
fn derive_escrow_pda(user_pubkey: &str, seed: &str) -> Result<String, AppError> {
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    let _user_pk = Pubkey::from_str(user_pubkey)
        .map_err(|_| AppError::InternalServerError("Invalid user public key for PDA derivation".into()))?;

    // Derive a PDA using the System Program as the program_id and the seed.
    // In production, this should use your deployed escrow program's ID.
    let escrow_program_id = Pubkey::from_str(
        &env::var("ESCROW_PROGRAM_ID")
            .unwrap_or_else(|_| "11111111111111111111111111111111".to_string())
    ).map_err(|_| AppError::InternalServerError("Invalid ESCROW_PROGRAM_ID".into()))?;

    let (pda, _bump) = Pubkey::find_program_address(
        &[seed.as_bytes()],
        &escrow_program_id,
    );

    Ok(pda.to_string())
}

pub async fn get_link(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;
    let link_id = path.into_inner();

    let link = PaymentLink::find_by_id(&mut conn, link_id)?
        .ok_or_else(|| AppError::BadRequest("Link not found".to_string()))?;

    // NEVER returns claim_hash or secret_code
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": link.status,
        "amount": link.amount,
        "token": link.token_mint,
        "memo": link.memo
    })))
}

pub async fn claim_link(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
    body: web::Json<ClaimRequest>
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;
    let link_id = path.into_inner();
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;

    // 1. find link
    let link = PaymentLink::find_by_id(&mut conn, link_id)?
        .ok_or_else(|| AppError::BadRequest("Link not found".to_string()))?;

    // 2. check status=funded (Active)
    if link.status != LinkStatus::Active {
        return Err(AppError::BadRequest("Link is not active or already claimed".to_string()));
    }

    // 3. Check expiry
    if link.expires_at < Utc::now() {
        return Err(AppError::BadRequest("Link has expired".to_string()));
    }

    // 4. DKG for new user if needed
    let dkg_conf = mpc_config()?;
    let _ = crate::services::dkg::generate_keypair(&dkg_conf, user_id).await;

    // 5. build escrow release tx
    let rpc_url = solana_rpc_url();
    let rpc_client = RpcClient::new(rpc_url);
    let unsigned_tx = build_transfer_tx(&rpc_client, &link.escrow_pda, &body.receiver_address, link.amount as u64).await?;

    // 6. sign via MPC
    let signed_buffer_b64 = coordinate_transaction_signature(&dkg_conf, user_id, unsigned_tx.as_bytes()).await?;
    let signed_tx = BASE64.decode(&signed_buffer_b64).unwrap_or(unsigned_tx.as_bytes().to_vec());

    // 7. submit to Solana
    let tx_hash = submit_transaction(&rpc_client, &String::from_utf8(signed_tx).unwrap_or_default()).await?;

    // 8. Finalize: unlock and deduct from creator's balance, mark link as claimed
    Balance::finalize_claim(&mut conn, link.creator_id, &link.token_mint, link.amount)?;
    PaymentLink::mark_as_claimed(&mut conn, link.id, user_id, &tx_hash)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "tx_hash": tx_hash
    })))
}
