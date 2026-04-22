use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Text, BigInt, SmallInt, Uuid as DieselUuid, Timestamptz};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::db::pool::DbPool;

#[derive(QueryableByName, Debug)]
pub struct TrackedAddress {
    #[diesel(sql_type = DieselUuid)]
    pub id: Uuid,
    #[diesel(sql_type = Text)]
    pub public_key: String,
}

#[derive(Debug, Clone)]
pub struct IndexedTransaction {
    pub user_id: Uuid,
    pub amount: i64,
    pub token_mint: String,
    pub token_symbol: String,
    pub tx_hash: String,
    pub tx_type: String,
    pub from_address: String,
    pub to_address: String,
    pub slot: i64,
    pub block_time: DateTime<Utc>,
}

pub fn load_tracked_addresses(pool: &DbPool) -> Vec<TrackedAddress> {
    let mut conn = pool.get().expect("Failed to get DB connection");

    sql_query("SELECT id, public_key FROM users WHERE is_active = true")
        .load::<TrackedAddress>(&mut conn)
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to load tracked addresses: {}", e);
            Vec::new()
        })
}

pub fn insert_transaction(pool: &DbPool, tx: &IndexedTransaction) {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("DB connection failed: {}", e);
            return;
        }
    };

    let result = sql_query(
        "INSERT INTO transactions (id, user_id, amount, token_mint, token_symbol, tx_hash, tx_type, from_address, to_address, slot, block_time)
         VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6::tx_type, $7, $8, $9, $10)
         ON CONFLICT (tx_hash) DO NOTHING"
    )
    .bind::<DieselUuid, _>(tx.user_id)
    .bind::<BigInt, _>(tx.amount)
    .bind::<Text, _>(&tx.token_mint)
    .bind::<Text, _>(&tx.token_symbol)
    .bind::<Text, _>(&tx.tx_hash)
    .bind::<Text, _>(&tx.tx_type)
    .bind::<Text, _>(&tx.from_address)
    .bind::<Text, _>(&tx.to_address)
    .bind::<BigInt, _>(tx.slot)
    .bind::<Timestamptz, _>(tx.block_time)
    .execute(&mut conn);

    match result {
        Ok(_) => tracing::info!("Indexed tx {} for user {}", tx.tx_hash, tx.user_id),
        Err(e) => tracing::error!("Failed to insert tx {}: {}", tx.tx_hash, e),
    }
}

pub fn credit_balance(pool: &DbPool, user_id: Uuid, mint: &str, symbol: &str, amount: i64, decimals: i16) {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("DB connection failed: {}", e);
            return;
        }
    };

    let result = sql_query(
        "INSERT INTO balances (id, user_id, token_mint, token_symbol, amount, available, locked, decimals, updated_at)
         VALUES (gen_random_uuid(), $1, $2, $3, $4, $4, 0, $5, NOW())
         ON CONFLICT (user_id, token_mint) DO UPDATE
         SET available = balances.available + $4,
             amount = balances.amount + $4,
             updated_at = NOW()"
    )
    .bind::<DieselUuid, _>(user_id)
    .bind::<Text, _>(mint)
    .bind::<Text, _>(symbol)
    .bind::<BigInt, _>(amount)
    .bind::<SmallInt, _>(decimals)
    .execute(&mut conn);

    match result {
        Ok(_) => tracing::info!("Credited {} lamports to user {} ({})", amount, user_id, mint),
        Err(e) => tracing::error!("Failed to credit balance: {}", e),
    }
}

pub fn debit_balance(pool: &DbPool, user_id: Uuid, mint: &str, amount: i64) {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("DB connection failed: {}", e);
            return;
        }
    };

    let result = sql_query(
        "UPDATE balances
         SET available = available - $3,
             amount = amount - $3,
             updated_at = NOW()
         WHERE user_id = $1 AND token_mint = $2 AND available >= $3"
    )
    .bind::<DieselUuid, _>(user_id)
    .bind::<Text, _>(mint)
    .bind::<BigInt, _>(amount)
    .execute(&mut conn);

    match result {
        Ok(_) => tracing::info!("Debited {} lamports from user {} ({})", amount, user_id, mint),
        Err(e) => tracing::error!("Failed to debit balance: {}", e),
    }
}
