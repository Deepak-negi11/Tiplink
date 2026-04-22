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
    tracing::info!("Starting TipLink Solana Indexer (Yellowstone gRPC)...");

    let cfg = IndexerConfig::from_env();
    let db_pool = pool::create_pool(&cfg.database_url);

    let tracked_accounts = TrackedAccounts::new();
    tracked_accounts.refresh_from_db(&db_pool);

    let tracked_accounts_clone = tracked_accounts.clone();
    let db_pool_clone = db_pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            tracked_accounts_clone.refresh_from_db(&db_pool_clone);
        }
    });

    let (tx_sender, mut tx_receiver) = mpsc::channel(1000);

   
    let stream_cfg = cfg.clone();
    let stream_accounts = tracked_accounts.clone();
    tokio::spawn(async move {
        let addrs = stream_accounts.all_addresses();
        stream::yellowstone::connect_and_stream(
            stream_cfg.grpc_endpoint,
            stream_cfg.grpc_token,
            addrs,
            tx_sender
        ).await;
    });

    
    while let Some(tx) = tx_receiver.recv().await {
        processor::process_transaction(&db_pool, &tracked_accounts, tx);
    }
}
