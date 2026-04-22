use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::swap_history;

#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, Serialize, Deserialize)]
#[ExistingTypePath = "crate::db::schema::sql_types::SwapStatus"]
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
    pub tx_hash: String,
    pub status: SwapStatus,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub requested_slippage_bps: i32, 
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
    pub tx_hash: &'a str,
    pub status: SwapStatus,
    pub requested_slippage_bps: i32,
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
    pub fn update_intent_status(
        conn: &mut PgConnection,
        intent_id: Uuid,
        new_status: &str,
        tx_hash: Option<&str>,
    ) -> QueryResult<usize> {
        diesel::update(transaction_intents::table.find(intent_id))
            .set((
                transaction_intents::status.eq(new_status),
                transaction_intents::final_tx_hash.eq(tx_hash),
            ))
            .execute(conn)
    }
}

