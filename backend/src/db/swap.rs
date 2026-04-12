use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::swap_history;

#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, Serialize, Deserialize)]
#[ExistingTypePath = "crate::db::schema::sql_types::SwapStatus"]
#[diesel(postgres_type(name = "swap_status"))]
pub enum SwapStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(Queryable, Selectable, Identifiable, Debug, Serialize, Deserialize)]
#[diesel(table_name = swap_history)]
pub struct SwapEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub input_mint: String,
    pub output_mint: String,
    pub output_amount: i64,
    pub input_amount: i64,
    pub fee_amount: i64,
    pub price_impact: BigDecimal,
    pub requested_slippage_bps: i32, 
    pub tx_hash: String,
    pub status: SwapStatus,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = swap_history)]
pub struct NewSwapEntry<'a> {
    pub user_id: Uuid,
    pub input_mint: &'a str,
    pub output_mint: &'a str,
    pub output_amount: i64,
    pub input_amount: i64,
    pub fee_amount: i64,
    pub price_impact: BigDecimal,
    pub requested_slippage_bps: i32, 
    pub tx_hash: &'a str,
    pub status: SwapStatus,
}

impl SwapEntry {
    /// Create a new swap history entry
    pub fn create_entry(
        conn: &mut PgConnection,
        new_entry: NewSwapEntry,
    ) -> QueryResult<SwapEntry> {
        diesel::insert_into(swap_history::table)
            .values(&new_entry)
            .get_result(conn)
    }

    /// Update the status of a swap
    pub fn update_status(
        conn: &mut PgConnection,
        swap_id: Uuid,
        new_status: SwapStatus,
    ) -> QueryResult<usize> {
        let now = if new_status == SwapStatus::Completed {
            Some(Utc::now())
        } else {
            None
        };

        diesel::update(swap_history::table.find(swap_id))
            .set((
                swap_history::status.eq(new_status),
                swap_history::confirmed_at.eq(now),
            ))
            .execute(conn)
    }

    /// Get swap history for a specific user
    pub fn get_user_history(
        conn: &mut PgConnection,
        user_id_val: Uuid,
    ) -> QueryResult<Vec<SwapEntry>> {
        swap_history::table
            .filter(swap_history::user_id.eq(user_id_val))
            .order(swap_history::created_at.desc())
            .select(SwapEntry::as_select())
            .load(conn)
    }
}

use crate::db::schema::transaction_intents;

#[derive(Queryable, Selectable, Identifiable, Debug, Serialize, Deserialize)]
#[diesel(table_name = transaction_intents)]
pub struct TransactionIntentEntry {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub intent_message: String,
    pub intent_signature: String,
    pub unsigned_payload: Option<String>,
    pub status: Option<String>,
    pub final_tx_hash: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = transaction_intents)]
pub struct NewTransactionIntent<'a> {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub intent_message: &'a str,
    pub intent_signature: &'a str,
    pub unsigned_payload: Option<&'a str>,
    pub status: Option<&'a str>,
}

impl TransactionIntentEntry {
    pub fn create_intent(
        conn: &mut PgConnection,
        new_entry: NewTransactionIntent,
    ) -> QueryResult<TransactionIntentEntry> {
        diesel::insert_into(transaction_intents::table)
            .values(&new_entry)
            .get_result(conn)
    }
}

// ------ EXPLICIT SWAP ROUTING (For Handlers and Indexers) ------

use crate::db::conn::DbPool;
use crate::error::AppError;

pub fn create(
    pool: &DbPool,
    user_id_val: Uuid,
    input_mint_val: &str,
    output_mint_val: &str,
    input_amount_val: i64,
    output_amount_val: i64,
    fee_amount_val: i64,
    tx_hash_val: &str
) -> Result<Uuid, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;
    
    let new_swap = NewSwapEntry {
        user_id: user_id_val,
        input_mint: input_mint_val,
        output_mint: output_mint_val,
        input_amount: input_amount_val,
        output_amount: output_amount_val,
        fee_amount: fee_amount_val,
        price_impact: BigDecimal::from(0), // Default mapping
        requested_slippage_bps: 0,
        tx_hash: tx_hash_val,
        status: SwapStatus::Pending,
    };

    let result: SwapEntry = diesel::insert_into(swap_history::table) // Ensure imports connect logically
        .values(&new_swap)
        .get_result(&mut conn)
        .map_err(|_| AppError::InternalServerError("Failed to create pending swap".into()))?;

    Ok(result.id)
}

pub fn confirm(
    pool: &DbPool,
    tx_hash_val: &str,
    output_amount_val: i64
) -> Result<(), AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;
    
    diesel::update(swap_history::table.filter(swap_history::tx_hash.eq(tx_hash_val)))
        .set((
            swap_history::status.eq(SwapStatus::Completed),
            swap_history::confirmed_at.eq(Some(Utc::now())),
            swap_history::output_amount.eq(output_amount_val),
        ))
        .execute(&mut conn)
        .map_err(|_| AppError::InternalServerError("Failed to confirm swap".into()))?;
        
    Ok(())
}

pub fn fail(
    pool: &DbPool,
    tx_hash_val: &str
) -> Result<(), AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;
    
    diesel::update(swap_history::table.filter(swap_history::tx_hash.eq(tx_hash_val)))
        .set(swap_history::status.eq(SwapStatus::Failed))
        .execute(&mut conn)
        .map_err(|_| AppError::InternalServerError("Failed to mark swap as failed".into()))?;
        
    Ok(())
}

pub fn get_history(
    pool: &DbPool,
    user_id_val: Uuid,
    limit_val: i64,
    offset_val: i64
) -> Result<Vec<SwapEntry>, AppError> {
    let mut conn = pool.get().map_err(|_| AppError::InternalServerError("Database connection failed".into()))?;
    
    let entries = swap_history::table
        .filter(swap_history::user_id.eq(user_id_val))
        .order(swap_history::created_at.desc())
        .limit(limit_val)
        .offset(offset_val)
        .load::<SwapEntry>(&mut conn)
        .map_err(|_| AppError::InternalServerError("Failed to load swap history".into()))?;
        
    Ok(entries)
}