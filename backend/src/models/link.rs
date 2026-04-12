use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateLinkRequest {
    pub amount: f64,
    pub token_mint: String,
}

#[derive(Debug, Deserialize)]
pub struct ClaimRequest {
    pub receiver_address: String,
}

#[derive(Debug, Serialize)]
pub struct LinkResponse {
    pub message: String,
    pub link_url: Option<String>,
}
