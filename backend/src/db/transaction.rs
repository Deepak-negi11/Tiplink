use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::transactions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, Serialize, Deserialize)]
#[ExistingTypePath = "crate::db::schema::sql_types::TxType"]
#[diesel(postgres_type(name = "tx_type"))]
pub enum TxType {
    Deposit,
    Withdrawal,
    Transfer,
    Swap,
}

#[derive(Queryable, Selectable, Identifiable, Debug, Serialize, Deserialize)]
#[diesel(table_name = transactions)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: i64,
    pub token_mint: String,
    pub token_symbol: String,
    pub tx_hash: String,
    pub tx_type: TxType,
    pub from_address: String,
    pub to_address: String,
    pub slot: i64,
    pub block_time: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = transactions)]
pub struct NewTransaction<'a> {
    pub user_id: Uuid,
    pub amount: i64,
    pub token_mint: &'a str,
    pub token_symbol: &'a str,
    pub tx_hash: &'a str,
    pub tx_type: TxType,
    pub from_address: &'a str,
    pub to_address: &'a str,
    pub slot: i64,
    pub block_time: DateTime<Utc>,
}

impl Transaction {
    /// Insert a new transaction record
    pub fn insert_tx(
        conn: &mut PgConnection,
        new_tx: NewTransaction,
    ) -> QueryResult<Transaction> {
        diesel::insert_into(transactions::table)
            .values(&new_tx)
            .get_result(conn)
    }

    /// Get transaction history for a user
    pub fn get_user_history(
        conn: &mut PgConnection,
        user_id_val: Uuid,
    ) -> QueryResult<Vec<Transaction>> {
        transactions::table
            .filter(transactions::user_id.eq(user_id_val))
            .order(transactions::block_time.desc())
            .select(Transaction::as_select())
            .load(conn)
    }

    /// Find a transaction by its hash
    pub fn find_by_hash(
        conn: &mut PgConnection,
        hash: &str,
    ) -> QueryResult<Option<Transaction>> {
        transactions::table
            .filter(transactions::tx_hash.eq(hash))
            .select(Transaction::as_select())
            .first(conn)
            .optional()
    }
}