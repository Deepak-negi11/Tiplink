use uuid::Uuid;
use chrono::Utc;
use crate::db::pool::DbPool;
use crate::db::queries::{self, IndexedTransaction};

pub fn handle_withdraw(
    pool: &DbPool,
    user_id: Uuid,
    amount: i64,
    mint: &str,
    symbol: &str,
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
        tx_type: "Withdrawal".to_string(),
        from_address: from_address.to_string(),
        to_address: to_address.to_string(),
        slot,
        block_time: Utc::now(),
    };

    queries::insert_transaction(pool, &tx);
    queries::debit_balance(pool, user_id, mint, amount);
}
