use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::prelude::*;
#[derive(Serialize,Deserialize,Debug)]
pub struct QuoteRequest{
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount:f64
}


#[derive(Serialize,Deserialize,Debug)]
pub struct QuoteResponse{
    pub input_mint:String,
    pub output_mint:String,
    pub input_amount:f64,
    pub output_amount:f64,
    pub price_impact_pct: f64, 
    pub fee_amount: f64,
}


#[derive(Serialize,Deserialize,Debug)]
pub struct SwapRequest{
    pub input_mint:String,
    pub output_amount:f64,
    pub amount:f64,
    pub output_mint:String,
    pub intent_signature:String,
    pub user_pubkey: String,
    pub requested_slippage_bps:u32

}

#[derive(Debug, Serialize)]
pub struct SwapResponse {
    pub message: String,
    pub tx_hash: String, 
}


#[derive(Serialize,Deserialize,Debug)]
pub struct BuildSwapResponse{
    pub intent_id:Uuid,
    pub unsigned_transaction :String,
}

#[derive(Debug, Deserialize)]
pub struct SubmitTxRequest {
    pub intent_id: Uuid, 
    pub signed_base64_tx: String, 
}


#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionIntent {
    pub id: Uuid,
    pub user_id: Option<Uuid>, 
    pub intent_message: String,
    pub intent_signature: String,
    pub unsigned_payload: Option<String>,
    pub status: Option<String>, 
    pub final_tx_hash: Option<String>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::db::schema::transaction_intents)]
pub struct NewTransactionIntent<'a> {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub intent_message: &'a str,
    pub intent_signature: &'a str,
    pub unsigned_payload: &'a str,
    pub status: &'a str,
}