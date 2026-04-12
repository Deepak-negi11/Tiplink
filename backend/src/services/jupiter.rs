// calls Jupiter quote + swap APIs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct QuoteParam {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    pub amount: u64,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JupiterQuote {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "otherAmountThreshold")]
    pub other_amount_threshold: String,
    #[serde(rename = "priceImpactPct")]
    pub price_impact_pct: String,
    pub route_plan: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct SwapRequest {
    #[serde(rename = "quoteResponse")]
    pub quote_response: JupiterQuote,
    #[serde(rename = "userPublicKey")]
    pub user_public_key: String,
    #[serde(rename = "wrapAndUnwrapSol")]
    pub wrap_and_unwrap_sol: bool,
}

#[derive(Debug, Deserialize)]
pub struct SwapResponse {
    #[serde(rename = "swapTransaction")]
    pub swap_transaction: String,  
}

pub async fn get_quote(
    input_mint: &str,
    output_mint: &str,
    amount: u64,
    slippage_bps: u16
) -> Result<JupiterQuote, AppError> {
    let client = Client::new();

    let url = format!(
        "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
        input_mint, output_mint, amount, slippage_bps
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|_| AppError::ExternalApi("Jupiter quote request failed".to_string()))?;

    if response.status().is_success() {
        let quote: JupiterQuote = response.json().await.map_err(|_| AppError::ExternalApi("Failed to parse Jupiter quote response".to_string()))?;
        Ok(quote)
    } else {
        Err(AppError::ExternalApi(format!("Jupiter quote API returned an error: {}", response.status())))
    }
}
