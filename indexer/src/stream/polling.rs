use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;
use crate::db::pool::DbPool;
use crate::db::queries;
use crate::filters::accounts::TrackedAccounts;

pub async fn start_polling(
    pool: DbPool,
    tracked_accounts: TrackedAccounts,
    rpc_url: String,
) {
    let client = Client::new();
    tracing::info!("Starting RPC Polling service (polling every 10s)...");

    loop {
        let addresses = tracked_accounts.all_addresses();
        for addr in addresses {
            if let Ok(balance) = get_sol_balance(&client, &rpc_url, &addr).await {
                if let Some(user_id) = tracked_accounts.get_user_id(&addr) {
                    sync_sol_balance(&pool, user_id, balance);
                }
            }

            let tokens = vec![
                ("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "USDC", 6),
                ("3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh", "WBTC", 8),
            ];

            for (mint, symbol, decimals) in tokens {
                if let Ok(balance) = get_token_balance(&client, &rpc_url, &addr, mint).await {
                     if let Some(user_id) = tracked_accounts.get_user_id(&addr) {
                        sync_token_balance(&pool, user_id, mint, symbol, balance, decimals);
                     }
                }
            }
        }

        sleep(Duration::from_secs(10)).await;
    }
}

async fn get_sol_balance(client: &Client, url: &str, address: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getBalance",
        "params": [address]
    });

    let res: Value = client.post(url).json(&payload).send().await?.json().await?;
    let balance = res["result"]["value"].as_u64().ok_or("Invalid balance response")?;
    Ok(balance)
}

async fn get_token_balance(client: &Client, url: &str, address: &str, mint: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTokenAccountsByOwner",
        "params": [
            address,
            { "mint": mint },
            { "encoding": "jsonParsed" }
        ]
    });

    let res: Value = client.post(url).json(&payload).send().await?.json().await?;
    let accounts = res["result"]["value"].as_array();
    
    if let Some(accounts) = accounts {
        if let Some(acc) = accounts.first() {
            let amount_str = acc["account"]["data"]["parsed"]["info"]["tokenAmount"]["amount"].as_str().unwrap_or("0");
            return Ok(amount_str.parse().unwrap_or(0));
        }
    }
    
    Ok(0)
}

fn sync_sol_balance(pool: &DbPool, user_id: uuid::Uuid, new_balance: u64) {
    let mut conn = pool.get().expect("DB connection failed");
    use diesel::prelude::*;
    use diesel::sql_query;
    use diesel::sql_types::{Uuid as DieselUuid, Text, BigInt, SmallInt};

    let _ = sql_query(
        "INSERT INTO balances (id, user_id, token_mint, token_symbol, amount, available, locked, decimals, updated_at)
         VALUES (gen_random_uuid(), $1, 'So11111111111111111111111111111111111111112', 'SOL', $2, $2, 0, 9, NOW())
         ON CONFLICT (user_id, token_mint) DO UPDATE
         SET amount = $2, available = $2 - balances.locked, updated_at = NOW()"
    )
    .bind::<DieselUuid, _>(user_id)
    .bind::<BigInt, _>(new_balance as i64)
    .execute(&mut conn);
}

fn sync_token_balance(pool: &DbPool, user_id: uuid::Uuid, mint: &str, symbol: &str, new_balance: u64, decimals: i16) {
    let mut conn = pool.get().expect("DB connection failed");
    use diesel::prelude::*;
    use diesel::sql_query;
    use diesel::sql_types::{Uuid as DieselUuid, Text, BigInt, SmallInt};

    let _ = sql_query(
        "INSERT INTO balances (id, user_id, token_mint, token_symbol, amount, available, locked, decimals, updated_at)
         VALUES (gen_random_uuid(), $1, $2, $3, $4, $4, 0, $5, NOW())
         ON CONFLICT (user_id, token_mint) DO UPDATE
         SET amount = $4, available = $4 - balances.locked, updated_at = NOW()"
    )
    .bind::<DieselUuid, _>(user_id)
    .bind::<Text, _>(mint)
    .bind::<Text, _>(symbol)
    .bind::<BigInt, _>(new_balance as i64)
    .bind::<SmallInt, _>(decimals)
    .execute(&mut conn);
}
