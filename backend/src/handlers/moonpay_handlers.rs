use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use crate::error::AppError;
use crate::db::conn::DbPool;
use crate::db::user::User;
use crate::db::balances::{Balance, NewBalance};
use crate::db::transaction::{NewTransaction, Transaction, TxType};
use crate::services::moonpay;

// ─── Config helpers ──────────────────────────────────────────

fn moonpay_publishable_key() -> Result<String, AppError> {
    std::env::var("MOONPAY_PUBLISHABLE_KEY")
        .map_err(|_| AppError::InternalServerError("MOONPAY_PUBLISHABLE_KEY not set".into()))
}

fn moonpay_secret_key() -> Result<String, AppError> {
    std::env::var("MOONPAY_SECRET_KEY")
        .map_err(|_| AppError::InternalServerError("MOONPAY_SECRET_KEY not set".into()))
}

fn moonpay_webhook_secret() -> Result<String, AppError> {
    std::env::var("MOONPAY_WEBHOOK_SECRET")
        .map_err(|_| AppError::InternalServerError("MOONPAY_WEBHOOK_SECRET not set".into()))
}

// ─── Sign URL ────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SignUrlRequest {
    pub currency_code: String,
    pub base_currency_amount: f64,
    pub base_currency_code: String,
}

#[derive(Serialize)]
pub struct SignUrlResponse {
    pub signed_url: String,
    pub limits: moonpay::MoonPayLimit,
}

pub async fn sign_url(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    body: web::Json<SignUrlRequest>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| AppError::Unauthorized("Not logged in".to_string()))?;

    let user = User::find_by_id(&mut conn, user_id)?
        .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

    let pub_key = moonpay_publishable_key()?;
    let secret_key = moonpay_secret_key()?;

    let widget_url = moonpay::build_widget_url(
        &pub_key,
        &user.public_key,
        &body.currency_code,
        &body.base_currency_code,
        body.base_currency_amount,
        "https://buy-sandbox.moonpay.com",  // Switch to "https://buy.moonpay.com" in production
    );

    let signed_url = moonpay::sign_url(&widget_url, &secret_key)?;

    let limits = moonpay::get_currency_limit(&pub_key, &body.currency_code).await?;

    Ok(HttpResponse::Ok().json(SignUrlResponse { signed_url, limits }))
}

// ─── Get Quote ───────────────────────────────────────────────

#[derive(Deserialize)]
pub struct QuoteQuery {
    pub currency_code: String,
    pub fiat_currency: String,
    pub fiat_amount: f64,
}

pub async fn get_quote(
    _req: HttpRequest,
    query: web::Query<QuoteQuery>,
) -> Result<HttpResponse, AppError> {
    let pub_key = moonpay_publishable_key()?;

    let quote = moonpay::get_buy_quote(
        &pub_key,
        &query.currency_code,
        &query.fiat_currency,
        query.fiat_amount,
    )
    .await?;

    Ok(HttpResponse::Ok().json(quote))
}

// ─── Get Limits ──────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LimitsQuery {
    pub currency_code: String,
}

pub async fn get_limits(
    _req: HttpRequest,
    query: web::Query<LimitsQuery>,
) -> Result<HttpResponse, AppError> {
    let pub_key = moonpay_publishable_key()?;

    let limits = moonpay::get_currency_limit(&pub_key, &query.currency_code).await?;

    Ok(HttpResponse::Ok().json(limits))
}

// ─── Webhook ─────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
pub struct MoonPayTransaction {
    pub id: String,
    pub status: String,
    #[serde(rename = "walletAddress")]
    pub wallet_address: String,
    #[serde(rename = "currencyCode")]
    pub currency_code: String,
    #[serde(rename = "baseCurrencyCode")]
    pub base_currency_code: String,
    #[serde(rename = "quoteCurrencyAmount")]
    pub quote_currency_amount: f64,
    #[serde(rename = "cryptoTransactionId")]
    pub tx_hash: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct MoonPayWebhookEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: MoonPayTransaction,
}

pub async fn webhook(
    req: HttpRequest,
    body: web::Bytes,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, AppError> {
    let webhook_secret = moonpay_webhook_secret()?;

    // 1. Verify signature
    let signature = req
        .headers()
        .get("Moonpay-Signature-v2")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing webhook signature".into()))?;

    if !moonpay::verify_webhook(&body, signature, &webhook_secret) {
        tracing::warn!("Invalid MoonPay webhook signature");
        return Err(AppError::Unauthorized("Invalid webhook signature".into()));
    }

    // 2. Parse event
    let event: MoonPayWebhookEvent = serde_json::from_slice(&body)
        .map_err(|e| AppError::BadRequest(format!("Invalid webhook body: {}", e)))?;

    tracing::info!(
        "MoonPay webhook: type={}, status={}",
        event.event_type,
        event.data.status
    );

    // 3. Only process completed transactions
    if event.data.status != "completed" {
        return Ok(HttpResponse::Ok().json(serde_json::json!({ "received": true })));
    }

    // 4. Find the user by their wallet address
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("DB connection failed".into()))?;

    let user = User::find_by_public_key(&mut conn, &event.data.wallet_address)?;

    if let Some(user) = user {
        let tx_id = format!("moonpay_{}", event.data.id);

        // Idempotency check: skip if already processed
        if let Ok(Some(_)) = Transaction::find_by_hash(&mut conn, &tx_id) {
            tracing::info!("MoonPay transaction {} already processed", event.data.id);
            return Ok(HttpResponse::Ok().json(serde_json::json!({ "received": true })));
        }

        let lamports = (event.data.quote_currency_amount * 1_000_000_000.0) as i64;

        // Credit the user's SOL balance
        let new_bal = NewBalance {
            user_id: user.id,
            token_mint: "So11111111111111111111111111111111111111112",
            token_symbol: "SOL",
            amount: lamports,
            available: lamports,
            locked: 0,
            decimals: 9,
        };
        Balance::add_balance(&mut conn, new_bal)
            .map_err(|e| AppError::InternalServerError(format!("Failed to credit balance: {}", e)))?;

        // Record the transaction
        let new_tx = NewTransaction {
            user_id: user.id,
            amount: lamports,
            token_mint: "So11111111111111111111111111111111111111112",
            token_symbol: "SOL",
            tx_hash: &tx_id,
            tx_type: TxType::Deposit,
            from_address: "moonpay",
            to_address: &event.data.wallet_address,
            slot: 0,
            block_time: Utc::now(),
        };
        Transaction::insert_tx(&mut conn, new_tx)
            .map_err(|e| AppError::InternalServerError(format!("Failed to record transaction: {}", e)))?;

        tracing::info!("Credited {} lamports to user {} via MoonPay", lamports, user.id);
    } else {
        tracing::warn!(
            "MoonPay webhook for unknown wallet: {}",
            event.data.wallet_address
        );
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({ "received": true })))
}
