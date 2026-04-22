use reqwest::Client;
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
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

pub async fn get_swap_transaction(
    quote: JupiterQuote,
    user_pubkey: &str
) -> Result<Vec<u8>, AppError> {
    let client = Client::new();
    let url = "https://quote-api.jup.ag/v6/swap";
    
    let swap_req = SwapRequest {
        quote_response: quote,
        user_public_key: user_pubkey.to_string(),
        wrap_and_unwrap_sol: true,
    };
    
    let response = client
        .post(url)
        .json(&swap_req)
        .send()
        .await
        .map_err(|_| AppError::ExternalApi("Failed to request Jupiter swap endpoint".to_string()))?;
        
    if response.status().is_success() {
        let swap_res: SwapResponse = response.json().await
            .map_err(|_| AppError::ExternalApi("Failed to parse Jupiter swap payload".to_string()))?;
            
        let decoded_tx_bytes = BASE64.decode(&swap_res.swap_transaction)
            .map_err(|_| AppError::ExternalApi("Failed to decode Jupiter base64 transaction string".to_string()))?;
            
        Ok(decoded_tx_bytes)
    } else {
        Err(AppError::ExternalApi(format!("Jupiter swap API returned an error: {}", response.status())))
    }
}

pub fn calculate_fee(amount: u64, fee_bps: u16) -> (u64, u64) {
    // Math bounds shifted to u128 safely preventing any extreme transaction u64 overflow
    let fee_amount = (amount as u128 * fee_bps as u128 / 10000) as u64;
    let amount_after_fee = amount - fee_amount;
    
    (amount_after_fee, fee_amount)
}
