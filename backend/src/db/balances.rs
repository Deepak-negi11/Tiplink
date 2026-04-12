use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::balances;

#[derive(Queryable, Selectable, Identifiable, Debug, Serialize, Deserialize)]
#[diesel(table_name = balances)]
pub struct Balance {
    pub id: Uuid,
    pub amount: i64,
    pub user_id: Uuid,
    pub token_mint: String,
    pub token_symbol: String,
    pub locked: i64,
    pub available: i64,
    pub decimals: i16,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = balances)]
pub struct NewBalance<'a> {
    pub user_id: Uuid,
    pub token_mint: &'a str,
    pub token_symbol: &'a str,
    pub amount: i64,
    pub available: i64,
    pub locked: i64,
    pub decimals: i16,
}

impl Balance {
    /// Retrieve all balances for a specific user
    pub fn get_user_balances(
        conn: &mut PgConnection,
        user_id_val: Uuid,
    ) -> QueryResult<Vec<Balance>> {
        balances::table
            .filter(balances::user_id.eq(user_id_val))
            .select(Balance::as_select())
            .load(conn)
    }

    /// Retrieve a specific token balance for a user
    pub fn get_token_balance(
        conn: &mut PgConnection,
        user_id_val: Uuid,
        mint: &str,
    ) -> QueryResult<Option<Balance>> {
        balances::table
            .filter(balances::user_id.eq(user_id_val))
            .filter(balances::token_mint.eq(mint))
            .select(Balance::as_select())
            .first(conn)
            .optional()
    }

    /// Lock funds for a payment link (moves funds from available to locked)
    pub fn lock_funds(
        conn: &mut PgConnection,
        user_id_val: Uuid,
        mint: &str,
        amount_to_lock: i64,
    ) -> QueryResult<usize> {
        diesel::update(
            balances::table
                .filter(balances::user_id.eq(user_id_val))
                .filter(balances::token_mint.eq(mint))
                .filter(balances::available.ge(amount_to_lock)),
        )
        .set((
            balances::available.eq(balances::available - amount_to_lock),
            balances::locked.eq(balances::locked + amount_to_lock),
            balances::updated_at.eq(Utc::now()),
        ))
        .execute(conn)
    }

    /// Finalize a claim (permanently removes funds from locked and total amount)
    pub fn finalize_claim(
        conn: &mut PgConnection,
        user_id_val: Uuid,
        mint: &str,
        amount_to_claim: i64,
    ) -> QueryResult<usize> {
        diesel::update(
            balances::table
                .filter(balances::user_id.eq(user_id_val))
                .filter(balances::token_mint.eq(mint))
                .filter(balances::locked.ge(amount_to_claim)),
        )
        .set((
            balances::locked.eq(balances::locked - amount_to_claim),
            balances::amount.eq(balances::amount - amount_to_claim),
            balances::updated_at.eq(Utc::now()),
        ))
        .execute(conn)
    }

    /// Refund locked funds back to available (e.g., link expired or cancelled)
    pub fn refund_locked_funds(
        conn: &mut PgConnection,
        user_id_val: Uuid,
        mint: &str,
        amount_to_refund: i64,
    ) -> QueryResult<usize> {
        diesel::update(
            balances::table
                .filter(balances::user_id.eq(user_id_val))
                .filter(balances::token_mint.eq(mint))
                .filter(balances::locked.ge(amount_to_refund)),
        )
        .set((
            balances::locked.eq(balances::locked - amount_to_refund),
            balances::available.eq(balances::available + amount_to_refund),
            balances::updated_at.eq(Utc::now()),
        ))
        .execute(conn)
    }

    /// Upsert balance (used for deposits or internal transfers)
    pub fn add_balance(
        conn: &mut PgConnection,
        new_balance: NewBalance,
    ) -> QueryResult<usize> {
        diesel::insert_into(balances::table)
            .values(&new_balance)
            .on_conflict((balances::user_id, balances::token_mint))
            .do_update()
            .set((
                balances::available.eq(balances::available + new_balance.amount),
                balances::amount.eq(balances::amount + new_balance.amount),
                balances::updated_at.eq(Utc::now()),
            ))
            .execute(conn)
    }

    /// Deduct funds from available (e.g., for withdrawals or swaps)
    pub fn subtract_balance(
        conn: &mut PgConnection,
        user_id_val: Uuid,
        mint: &str,
        amount_to_subtract: i64,
    ) -> QueryResult<usize> {
        diesel::update(
            balances::table
                .filter(balances::user_id.eq(user_id_val))
                .filter(balances::token_mint.eq(mint))
                .filter(balances::available.ge(amount_to_subtract)),
        )
        .set((
            balances::available.eq(balances::available - amount_to_subtract),
            balances::amount.eq(balances::amount - amount_to_subtract),
            balances::updated_at.eq(Utc::now()),
        ))
        .execute(conn)
    }
}
