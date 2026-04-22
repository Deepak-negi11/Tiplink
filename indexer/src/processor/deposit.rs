use uuid::Uuid;
use chrono::Utc;
use crate::db::pool::DbPool;
use crate::db::queries::{self, IndexedTransaction};

pub fn handle_deposit(
    pool: &DbPool,
    user_id: Uuid,
    amount: i64,
    mint: &str,
    symbol: &str,
    decimals: i16,
    tx_hash: &str,
    from_address: &str,
    to_address: &str,
    slot: i64,
) {
    let tx = IndexedTransaction {
        user_id,
        amount,
        token_mint: mint.to_string(),
        token_symbol: symbol.to_string(),
        tx_hash: tx_hash.to_string(),
        tx_type: "Deposit".to_string(),
        from_address: from_address.to_string(),
        to_address: to_address.to_string(),
        slot,
        block_time: Utc::now(),
    };

    
    queries::insert_transaction(pool, &tx);

    queries::credit_balance(pool, user_id, mint, symbol, amount, decimals);
}
