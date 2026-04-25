use tokio::sync::mpsc;

mod config;
mod db;
mod filters;
mod processor;
mod stream;

use config::IndexerConfig;
use db::pool;
use filters::accounts::TrackedAccounts;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting TipLink Solana Indexer (RPC Polling)...");

    let cfg = IndexerConfig::from_env();
    let db_pool = pool::create_pool(&cfg.database_url);

    let tracked_accounts = TrackedAccounts::new();
    tracked_accounts.refresh_from_db(&db_pool);

    let tracked_accounts_clone = tracked_accounts.clone();
    let db_pool_clone = db_pool.clone();
    let refresh_pool = db_pool_clone.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            tracked_accounts_clone.refresh_from_db(&refresh_pool);
        }
    });

    let (tx_sender, mut tx_receiver) = mpsc::channel(1000);

   
    let rpc_cfg = cfg.clone();
    let rpc_accounts = tracked_accounts.clone();
    let rpc_pool = db_pool_clone.clone();
    tokio::spawn(async move {
        stream::polling::start_polling(
            rpc_pool,
            rpc_accounts,
            rpc_cfg.solana_rpc_url, // Use the standard RPC URL from config
        ).await;
    });

    
    while let Some(tx) = tx_receiver.recv().await {
        processor::process_transaction(&db_pool, &tracked_accounts, tx);
    }
}
