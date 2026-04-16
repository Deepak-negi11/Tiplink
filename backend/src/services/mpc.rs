use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;
use std::collections::BTreeMap;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tokio::sync::mpsc;
use frost_ed25519::{aggregate, Identifier, SignatureShare, SigningPackage};
use frost_ed25519::keys::PublicKeyPackage;
use bs58;

use crate::error::AppError;
use crate::services::dkg::Config;

// api key is our secret in this and the body is the message
pub fn hmac_sign(body: &str, api_key: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(api_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(body.as_bytes());
    let result = mac.finalize();
    format!("{:x}", result.into_bytes())
}


async fn call_mpc_node(
    client: &Client,
    url: &str,
    endpoint: &str, 
    session: Uuid, 
    user: Uuid, 
    payload: Value, 
    api_key: &str
) -> Result<Value, AppError> {
    let body_str = json!({ "session_id": session, "user_id": user, "data": payload }).to_string();
    let signature = hmac_sign(&body_str, api_key);

    let res = client.post(format!("{}{}", url, endpoint))
        .header("X-Signature", signature)
        .header("Content-Type", "application/json")
        .body(body_str)
        .send().await
        .map_err(|_| AppError::ExternalApi("MPC connection failed".to_string()))?
        .json().await
        .map_err(|_| AppError::ExternalApi("MPC payload parsing failed".to_string()))?;
    Ok(res)
}


pub async fn init_signing(client: &Client, url: &str, api_key: &str, session: Uuid, user: Uuid, tx_bytes: &[u8]) -> Result<(), AppError> {
    let b64_tx = base64::encode(tx_bytes);
    call_mpc_node(client, url, "/sign/init", session, user, json!({"tx": b64_tx}), api_key).await?;
    Ok(())
}

/// Round 1: Fetches nonce commitment hash
pub async fn get_commitment(client: &Client, url: &str, api_key: &str, session: Uuid, user: Uuid) -> Result<String, AppError> {
    let res = call_mpc_node(client, url, "/sign/round1", session, user, json!({}), api_key).await?;
    res["commitment"].as_str().map(|s| s.to_string()).ok_or_else(|| AppError::ExternalApi("Missing commitment shard".into()))
}

/// Round 2: Fetches mathematical signature combination
pub async fn get_partial_sig(client: &Client, url: &str, api_key: &str, session: Uuid, user: Uuid, commitments: Value) -> Result<String, AppError> {
    let res = call_mpc_node(client, url, "/sign/round2", session, user, json!({"commitments": commitments}), api_key).await?;
    res["partial_sig"].as_str().map(|s| s.to_string()).ok_or_else(|| AppError::ExternalApi("Missing partial_sig shard".into()))
}

pub fn combine_signatures(
    signing_package:&SignaturePackage,
    share1:SignatureShare,
    id1:Identifier,
    share2:SignatureShare,
    id2:Identifier,
    pubkey_package:&PublicKeyPackage,
) -> Result<String, AppError> {

    let mut signature_share = BTreeMap::new();
    signature_share.insert(id1, share1);
    signature_share.insert(id2, share2);

    let final_signature = aggregate(
        signing_package,
        &signature_share,
        pubkey_package
    ).map_err(|_| AppError::InternalServerError("Cryptographic aggregation failed:invalid share".to_string()))?;

   let sig_bytes = final_signature.to_bytes();
   let solana_base58_sig = bs58::encode(sig_bytes).into_string();
    
   Ok(solana_base58_sig)
}

pub async fn coordinate_transaction_signature(
    nodes: &Config,
    user_id: Uuid,
    tx_bytes: &[u8]
) -> Result<String, AppError> {
    let client = Client::new();
    let session_id = Uuid::new_v4();

    let mpc_configs = vec![
        (nodes.aws.clone(), std::env::var("INTERNAL_MPC_KEY").unwrap_or_default()),
        (nodes.do_ocean.clone(), std::env::var("INTERNAL_MPC_KEY").unwrap_or_default()),
        (nodes.cloudflare.clone(), std::env::var("INTERNAL_MPC_KEY").unwrap_or_default()),
    ];

  
    for (url, key) in &mpc_configs {
        let _ = init_signing(&client, url, key, session_id, user_id, tx_bytes).await;
    }

    let (tx, mut rx) = mpsc::channel(3);
    for (id, (url, key)) in mpc_configs.iter().enumerate() {
        let client_c = client.clone();
        let url_c = url.clone();
        let key_c = key.clone();
        let tx_c = tx.clone();
        
        tokio::spawn(async move {
            if let Ok(comm) = get_commitment(&client_c, &url_c, &key_c, session_id, user_id).await {
                let _ = tx_c.send((id, comm)).await;
            }
        });
    }

    let mut commitments = BTreeMap::new();
    while let Some((id, comm)) = rx.recv().await {
        commitments.insert(id.to_string(), comm);
        if commitments.len() == 2 {
            break; // THRESHOLD REACHED
        }
    }
    
    if commitments.len() < 2 {
        return Err(AppError::ExternalApi("Threshold not reached for Round 1 (Multiple node failures)".to_string()));
    }
    let comms_value = json!(commitments);

    let (tx2, mut rx2) = mpsc::channel(3);
    for (id, (url, key)) in mpc_configs.iter().enumerate() {
        let client_c = client.clone();
        let url_c = url.clone();
        let key_c = key.clone();
        let tx2_c = tx2.clone();
        let comms_val = comms_value.clone();
        
        tokio::spawn(async move {
            if let Ok(psig) = get_partial_sig(&client_c, &url_c, &key_c, session_id, user_id, comms_val).await {
                let _ = tx2_c.send((id, psig)).await;
            }
        });
    }

    let mut partial_sigs = Vec::new();
    while let Some((_, psig)) = rx2.recv().await {
        partial_sigs.push(psig);
        if partial_sigs.len() == 2 {
            break; 
        }
    }

    if partial_sigs.len() < 2 {
        return Err(AppError::ExternalApi("Threshold not reached for Round 2 (Multiple node failures)".to_string()));
    }

    let share1_bytes = hex::decode(&partial_sigs[0])
        .map_err(|_| AppError::InternalServerError("Failed to decode share 1".into()))?;
    let share2_bytes = hex::decode(&partial_sigs[1])
        .map_err(|_| AppError::InternalServerError("Failed to decode share 2".into()))?;

    
    let share1 = SignatureShare::deserialize(&share1_bytes)
        .map_err(|_| AppError::InternalServerError("Invalid math share 1".into()))?;
    let share2 = SignatureShare::deserialize(&share2_bytes)
        .map_err(|_| AppError::InternalServerError("Invalid math share 2".into()))?;

    let id1 = Identifier::try_from(1u16).unwrap(); 
    let id2 = Identifier::try_from(2u16).unwrap();

    
    let final_sig = combine_signatures(&signing_package , share1, id1 , share2,id2 ,&pubkey_package);
    Ok(final_sig)
}