use hmac::{Mac, Hmac};
use reqwest::Client;
use sha2::Sha256;
use serde::{Deserialize, Serialize};
use base64::Engine;
use crate::error::AppError;

const MOONPAY_BASE_URL: &str = "https://api.moonpay.com";

/// HMAC-sign a MoonPay widget URL so it can't be tampered with.
pub fn sign_url(
    original_url: &str,
    secret_key: &str,
) -> Result<String, AppError> {
    let parsed = url::Url::parse(original_url)
        .map_err(|_| AppError::InternalServerError("Invalid URL".into()))?;

    let query_string = parsed.query().unwrap_or("");

    let mut mac = Hmac::<Sha256>::new_from_slice(secret_key.as_bytes())
        .map_err(|_| AppError::InternalServerError("HMAC init failed".into()))?;
    mac.update(format!("?{}", query_string).as_bytes());

    let signature = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    let encoded_sig = urlencoding::encode(&signature);

    let signed_url = if query_string.is_empty() {
        format!("{}?signature={}", original_url, encoded_sig)
    } else {
        format!("{}&signature={}", original_url, encoded_sig)
    };

    Ok(signed_url)
}

/// Build the MoonPay buy-widget URL with pre-filled parameters.
pub fn build_widget_url(
    publishable_key: &str,
    wallet_address: &str,
    currency_code: &str,
    base_currency_code: &str,
    base_currency_amount: f64,
    base_url: &str,
) -> String {
    let amount_str = base_currency_amount.to_string();
    let params = [
        ("apiKey", publishable_key),
        ("currencyCode", currency_code),
        ("baseCurrencyCode", base_currency_code),
        ("baseCurrencyAmount", &amount_str),
        ("walletAddress", wallet_address),
        ("network", "solana"),
        ("showWalletAddressForm", "false"),
        ("redirectURL", "https://localhost:3000/deposit/success"),
    ];

    let query: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    format!("{}?{}", base_url, query)
}

#[derive(Deserialize, Debug, Clone)]
pub struct MoonPayLimitCurrency {
    #[serde(rename = "minBuyAmount", default)]
    pub min_buy_amount: f64,
    #[serde(rename = "maxBuyAmount", default)]
    pub max_buy_amount: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MoonPayLimitRaw {
    #[serde(rename = "baseCurrency")]
    pub base_currency: MoonPayLimitCurrency,
}

/// The shape we send to the frontend.
#[derive(Serialize, Debug, Clone)]
pub struct MoonPayLimit {
    #[serde(rename = "baseCurrencyMinBuyAmount")]
    pub base_min_amount: f64,
    #[serde(rename = "baseCurrencyMaxBuyAmount")]
    pub base_max_amount: f64,
}

/// Fetch min/max purchase limits for a given crypto currency.
pub async fn get_currency_limit(
    api_key: &str,
    currency_code: &str,
) -> Result<MoonPayLimit, AppError> {
    let client = Client::new();
    let url = format!(
        "{}/v3/currencies/{}/limits?apiKey={}&baseCurrencyCode=usd",
        MOONPAY_BASE_URL, currency_code, api_key
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::ExternalApi(e.to_string()))?;

    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(AppError::ExternalApi(format!("MoonPay limit error: {}", err_text)));
    }

    let raw = resp.json::<MoonPayLimitRaw>()
        .await
        .map_err(|e| AppError::ExternalApi(format!("MoonPay limit parse error: {}", e)))?;

    Ok(MoonPayLimit {
        base_min_amount: raw.base_currency.min_buy_amount,
        base_max_amount: raw.base_currency.max_buy_amount,
    })
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MoonPayQuote {
    #[serde(rename = "baseCurrencyAmount", default)]
    pub base_currency_amount: f64,
    #[serde(rename = "quoteCurrencyAmount", default)]
    pub quote_currency_amount: f64,
    #[serde(rename = "feeAmount", default)]
    pub fee_amount: f64,
    #[serde(rename = "extraFeeAmount", default)]
    pub extra_fee_amount: f64,
    #[serde(rename = "totalAmount", default)]
    pub total_amount: f64,
}

/// Get a real-time buy quote from MoonPay.
pub async fn get_buy_quote(
    api_key: &str,
    currency_code: &str,
    base_currency_code: &str,
    base_currency_amount: f64,
) -> Result<MoonPayQuote, AppError> {
    let client = Client::new();
    let url = format!(
        "{}/v3/currencies/{}/buy_quote?apiKey={}&baseCurrencyCode={}&baseCurrencyAmount={}",
        MOONPAY_BASE_URL, currency_code, api_key, base_currency_code, base_currency_amount
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::ExternalApi(e.to_string()))?;

    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(AppError::ExternalApi(format!("MoonPay quote error: {}", err_text)));
    }

    resp.json::<MoonPayQuote>()
        .await
        .map_err(|e| AppError::ExternalApi(format!("MoonPay quote parse error: {}", e)))
}

/// Verify that a MoonPay webhook payload matches the expected HMAC signature.
pub fn verify_webhook(
    body: &[u8],
    signature_header: &str,
    webhook_secret: &str,
) -> bool {
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(webhook_secret.as_bytes()) else {
        return false;
    };
    mac.update(body);
    let expected = hex::encode(mac.finalize().into_bytes());

    // Constant-time comparison to prevent timing attacks
    if expected.len() != signature_header.len() {
        return false;
    }
    expected
        .as_bytes()
        .iter()
        .zip(signature_header.as_bytes().iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}