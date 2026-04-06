#get balance , lock , unlock , update

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;
use crate::db::schema::balances;


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


impl Balance {

    pub fn get_balance(
        conn:&mut PgConnection,
        user_id: Uuid
    )->QueryResult<Balance>{

        balances::table
            .filter(balances::user_id.eq(user_id))
            .select(Balance::as_select())
            .load::<Balance>(conn)
            

    }

    pub fn lock_funds(
        conn:&mut PgConnection,
        user_id_val:Uuid,
        amount_to_lock:i64,
        token_mint:String,
        token_sybol:String
    )->Result<(),String>{
        let updated_rows = diesel::update(
            balances::table
                .filter(balances::user_id.eq(user_id_val))
                .filter(balances::token_mint.eq(token_mint))
                .filter(balances::available.get(amount_to_lock))
        )
        .set((
            balances::available.eq(balances::available - amount_to_lock),
            balances::locked.eq(balances::locked + amount_to_lock)
        ))
        .execute(conn)
        .map_err(|e| e.to_string())?;

        if updated_rows == 0 {
            return Err("Insufficent available balance " .to_string())
        }
        Ok(())
    }

    pub fn unlock_funds(
        conn:&mut PgConnection,
        user_id_unlock:Uuid,
        token_mint:String,
        amount_to_claim:i64
    )-> Result<() , String>{

        let unlock = diesel::update(
            balances::table
                .filter(balances::user_id.eq(user_id_unlock))
                .filter(balances::token_mint.eq(token_mint))
                .filter(balances::locked.get(amount_to_claim))
        )
        .set((
            balances::available.eq(available + amount_to_claim),
            balances::locked.eq(locked - amount_to_claim)
        ))
        .execute(conn)
        .map_err(|e| e.to_string())?;
        if unlock == 0 {
            return Err("Insufficient locked balance".to_string());
        }
        Ok(())
    }

    pub fn update_balance(
        conn:PgConnection,
        amount
    )

}

