use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::geyser::{
    SubscribeRequest, SubscribeRequestFilterTransactions, SubscribeUpdateTransaction,
    subscribe_update::UpdateOneof,
};
use std::collections::HashMap;
use tokio::sync::mpsc;
use futures::{stream::StreamExt, SinkExt};

pub async fn connect_and_stream(
    endpoint: String,
    token: Option<String>,
    tracked_addresses: Vec<String>,
    tx_sender: mpsc::Sender<SubscribeUpdateTransaction>,
) {
    loop {
        tracing::info!("Connecting to Yellowstone gRPC at {}", endpoint);
        
        let mut client = match GeyserGrpcClient::build_from_shared(endpoint.clone()) {
            Ok(builder) => {
                let builder = if let Some(ref t) = token {
                    builder.x_token(Some(t.clone())).expect("Invalid token format")
                } else {
                    builder
                };
                match builder.connect().await {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::error!("Failed to connect to gRPC: {}. Retrying in 5s...", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        continue;
                    }
                }
            }
            Err(e) => {
                tracing::error!("Invalid gRPC endpoint building: {}", e);
                return;
            }
        };

        // If no addresses mapped yet, provide a dummy fallback
        let addrs = if tracked_addresses.is_empty() {
            vec!["11111111111111111111111111111111".to_string()]
        } else {
            tracked_addresses.clone()
        };

        let mut transactions_filter = HashMap::new();
        // Subscribe to all transactions involving the tracked addresses
        // This is why gRPC is efficient — the filtering happens on the Geyser sever,
        // we only receive network traffic directly correlated to our users!
        transactions_filter.insert(
            "tracked_wallets".to_string(),
            SubscribeRequestFilterTransactions {
                vote: Some(false),         // Ignore vote txs (exactly what you noticed!)
                failed: Some(false),       // Ignore failed txs
                signature: None,
                account_include: addrs,    // Only fetch for our DB users
                account_exclude: vec![],
                account_required: vec![],
            },
        );

        let request = SubscribeRequest {
            transactions: transactions_filter,
            ..Default::default()
        };

        let (mut subscribe_tx, mut stream) = match client.subscribe().await {
            Ok((tx, stream)) => (tx, stream),
            Err(e) => {
                tracing::error!("Failed to subscribe: {}. Retrying...", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        if let Err(e) = subscribe_tx.send(request).await {
            tracing::error!("Failed to send subscribe request: {}", e);
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        }

        tracing::info!("Successfully subscribed to Yellowstone stream.");

        // Read stream infinitely
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(update) => {
                    if let Some(UpdateOneof::Transaction(tx)) = update.update_oneof {
                        // Forward over tokio channel to the processor loop
                        if tx_sender.send(tx).await.is_err() {
                            tracing::error!("Transaction receiver channel closed. Exiting stream.");
                            return;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Stream error: {}. Reconnecting...", e);
                    break;
                }
            }
        }
        
        tracing::warn!("Stream disconnected. Reconnecting in 5s...");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
