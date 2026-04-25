use serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SendRequest {
    pub to_address: String,
    pub token_mint: String,
    pub amount: f64,
}

use chrono::{DateTime,Utc};

#[derive(Debug,Deserialize,Serialize)]
pub struct TransactionRequest{
    pub to_address:String,
    pub token_mint:String,
    pub amount :i64
}

#[derive(Debug,Deserialize,Serialize)]
pub struct TransactionResponse{
    pub message:String,
    pub tx_hash:String
}

#[derive(Debug,Deserialize,Serialize)]
pub struct BalanceResponse{
    pub token_mint:String,
    pub token_symbol:String,
    pub available:f64,
    pub locked:f64
}

#[derive(Debug,Deserialize,Serialize)]
pub struct TransactionHistoryResponse{
    pub tx_hash:String,
    pub tx_type:String,
    pub token_symbol:String,
    pub amount:f64,
    pub block_time:DateTime<Utc>
}

