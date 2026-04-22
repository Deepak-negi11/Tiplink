use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;
use std::collections::BTreeMap;
use tokio::sync::mpsc;
use frost_ed25519::{self as frost, Identifier, round2::SignatureShare};

use crate::error::AppError;
use crate::services::dkg::Config;
use crate::services::hmac::{post_to_node, post_to_node_with_session};

async fn init_signing(
    client: &Client,
    url: &str,
    api_key: &str,
    session: Uuid,
    user: Uuid,
    tx_bytes: &[u8],
) -> Result<(), AppError> {
    let b64_tx = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        tx_bytes,
    );
    post_to_node_with_session(
        client, url, "/sign/init", session, user,
        json!({"tx": b64_tx}), api_key,
    ).await?;
    Ok(())
}

async fn get_commitment(
    client: &Client,
    url: &str,
    api_key: &str,
    session: Uuid,
    user: Uuid,
) -> Result<Value, AppError> {
    let body_str = json!({ "session_id": session, "user_id": user }).to_string();
    let res = post_to_node(client, url, "/sign/round1", &body_str, api_key).await?;

    let commitment = res.get("commitment")
        .ok_or_else(|| AppError::ExternalApi("Missing commitment from sign round1".into()))?;

    Ok(commitment.clone())
}

async fn get_partial_sig(
    client: &Client,
    url: &str,
    api_key: &str,
    session: Uuid,
    user: Uuid,
    commitments: Value,
) -> Result<String, AppError> {
    let res = post_to_node_with_session(
        client, url, "/sign/round2", session, user,
        json!({"commitments": commitments}), api_key,
    ).await?;
    res["partial_sig"].as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::ExternalApi("Missing partial_sig shard".into()))
}

/// Retrieves the PublicKeyPackage from any responding MPC node.
/// Each node stores the group public key package after DKG finalization.
async fn get_pubkey_package(
    client: &Client,
    config: &Config,
    user_id: Uuid,
    api_key: &str,
) -> Result<frost::keys::PublicKeyPackage, AppError> {
    // Try each node until one returns the stored pubkey package
    let node_urls = [&config.aws, &config.do_ocean, &config.cloudflare];
    
    for url in node_urls {
        let body_str = json!({ "user_id": user_id }).to_string();
        if let Ok(res) = post_to_node(client, url, "/keys/pubkey_package", &body_str, api_key).await {
            if let Some(pkg_val) = res.get("pubkey_package") {
                let pkg: frost::keys::PublicKeyPackage = serde_json::from_value(pkg_val.clone())
                    .map_err(|e| AppError::InternalServerError(
                        format!("Failed to deserialize PublicKeyPackage: {}", e)
                    ))?;
                return Ok(pkg);
            }
        }
    }

    Err(AppError::ExternalApi(
        "Failed to retrieve PublicKeyPackage from any MPC node. Signing cannot proceed.".into()
    ))
}

pub async fn coordinate_transaction_signature(
    nodes: &Config,
    user_id: Uuid,
    tx_bytes: &[u8],
) -> Result<Vec<u8>, AppError> {
    let client = Client::new();
    let session_id = Uuid::new_v4();
    let key = std::env::var("INTERNAL_MPC_KEY").unwrap_or_default();

    let node_urls = vec![
        (nodes.aws.clone(), 1u16),
        (nodes.do_ocean.clone(), 2u16),
        (nodes.cloudflare.clone(), 3u16),
    ];

    // Initialize signing sessions on all nodes
    for (url, _) in &node_urls {
        init_signing(&client, url, &key, session_id, user_id, tx_bytes).await?;
    }

    // Round 1: Collect signing commitments (threshold = 2 of 3)
    let (tx_comm, mut rx_comm) = mpsc::channel(3);
    for (node_idx, (url, _)) in node_urls.iter().enumerate() {
        let client_c = client.clone();
        let url_c = url.clone();
        let key_c = key.clone();
        let tx_c = tx_comm.clone();

        tokio::spawn(async move {
            if let Ok(comm) = get_commitment(&client_c, &url_c, &key_c, session_id, user_id).await {
                let _ = tx_c.send((node_idx, comm)).await;
            }
        });
    }
    drop(tx_comm);

    let mut commitments: BTreeMap<String, Value> = BTreeMap::new();
    let mut responding_nodes = Vec::new();
    while let Some((node_idx, comm)) = rx_comm.recv().await {
        let node_id = node_urls[node_idx].1;
        commitments.insert(node_id.to_string(), comm);
        responding_nodes.push(node_idx);
        if commitments.len() == 2 {
            break;
        }
    }

    if commitments.len() < 2 {
        return Err(AppError::ExternalApi("Threshold not reached for Round 1".to_string()));
    }
    let comms_value = json!(commitments);

    // Round 2: Collect partial signature shares (threshold = 2 of 3)
    let (tx_sig, mut rx_sig) = mpsc::channel(3);
    for &node_idx in &responding_nodes {
        let (url, node_id) = node_urls[node_idx].clone();
        let client_c = client.clone();
        let key_c = key.clone();
        let comms_val = comms_value.clone();
        let tx_sig_c = tx_sig.clone();

        tokio::spawn(async move {
            if let Ok(psig) = get_partial_sig(&client_c, &url, &key_c, session_id, user_id, comms_val).await {
                let _ = tx_sig_c.send((node_id, psig)).await;
            }
        });
    }
    drop(tx_sig);

    let mut partial_sigs: Vec<(u16, String)> = Vec::new();
    while let Some((node_id, psig)) = rx_sig.recv().await {
        partial_sigs.push((node_id, psig));
        if partial_sigs.len() == 2 {
            break;
        }
    }

    if partial_sigs.len() < 2 {
        return Err(AppError::ExternalApi("Threshold not reached for Round 2".to_string()));
    }

    // Reconstruct the signing commitments map for aggregation
    let mut frost_commitments: BTreeMap<Identifier, frost::round1::SigningCommitments> = BTreeMap::new();
    for (id_str, comm_val) in &commitments {
        let node_id: u16 = id_str.parse()
            .map_err(|_| AppError::InternalServerError("Invalid node id in commitments".into()))?;
        let identifier: Identifier = node_id.try_into()
            .map_err(|_| AppError::InternalServerError("Invalid FROST identifier".into()))?;
        let comm: frost::round1::SigningCommitments = serde_json::from_value(comm_val.clone())
            .map_err(|e| AppError::InternalServerError(format!("Failed to parse commitment: {}", e)))?;
        frost_commitments.insert(identifier, comm);
    }

    let signing_package = frost::SigningPackage::new(frost_commitments, tx_bytes);

    // Deserialize partial signature shares
    let mut signature_shares: BTreeMap<Identifier, SignatureShare> = BTreeMap::new();
    for (node_id, psig_hex) in &partial_sigs {
        let share_bytes = hex::decode(psig_hex)
            .map_err(|_| AppError::InternalServerError(format!("Failed to decode share from node {}", node_id)))?;
        let share = SignatureShare::deserialize(&share_bytes)
            .map_err(|_| AppError::InternalServerError(format!("Invalid signature share from node {}", node_id)))?;
        let identifier: Identifier = (*node_id).try_into()
            .map_err(|_| AppError::InternalServerError("Invalid FROST identifier".into()))?;
        signature_shares.insert(identifier, share);
    }

    // Retrieve the group PublicKeyPackage from MPC nodes
    let pubkey_package = get_pubkey_package(&client, nodes, user_id, &key).await?;

    // REAL aggregation: combine partial signatures into a full group signature
    let group_signature = frost::aggregate(&signing_package, &signature_shares, &pubkey_package)
        .map_err(|e| AppError::InternalServerError(
            format!("FROST signature aggregation failed: {}. This may indicate a corrupted shard or node mismatch.", e)
        ))?;

    Ok(group_signature.serialize().to_vec())
}