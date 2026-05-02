use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;
use std::collections::BTreeMap;
use std::time::Instant;
use frost_ed25519 as frost;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use crate::state::{MpcState, SignSession};
use crate::middleware;
use crate::vault;
use crate::crypto;
use crate::error::MpcError;
use crate::util::parse_identifier;



#[derive(Deserialize)]
pub struct SignInitRequest {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub data: SignInitData,
}

#[derive(Deserialize)]
pub struct SignInitData {
    pub tx: String,
}

#[derive(Deserialize)]
pub struct SignRound1Request {
    pub session_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Deserialize)]
pub struct SignRound2Request {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub data: SignRound2Data,
}

#[derive(Deserialize)]
pub struct SignRound2Data {
    pub commitments: BTreeMap<String, serde_json::Value>,
}

pub async fn sign_init(
    req: HttpRequest,
    body_bytes: web::Bytes,
    server_state: web::Data<MpcState>,
) -> Result<HttpResponse, MpcError> {
    if !middleware::is_authentic(&req, &body_bytes, &server_state.hmac_secret) {
        return Err(MpcError::Unauthorised("HMAC authentication failed".to_string()));
    }

    let payload: SignInitRequest = serde_json::from_slice(&body_bytes)
        .map_err(|e| MpcError::BadRequest(format!("Invalid JSON: {}", e)))?;

    let tx_bytes = BASE64.decode(&payload.data.tx)
        .map_err(|_| MpcError::BadRequest("Invalid base64 transaction payload".to_string()))?;

    let (key_pkg_bytes, pubkey_pkg_bytes) = vault::load_key_package(
        payload.user_id,
        server_state.node_id as i32,
        &server_state.aes_secret_key,
        &server_state.db_pool,
    ).await?;

    let key_package: frost::keys::KeyPackage = serde_json::from_slice(&key_pkg_bytes)
        .map_err(|e| MpcError::Internal(format!("Failed to deserialize key package: {}", e)))?;
    let pubkey_package: frost::keys::PublicKeyPackage = serde_json::from_slice(&pubkey_pkg_bytes)
        .map_err(|e| MpcError::Internal(format!("Failed to deserialize pubkey package: {}", e)))?;

    let session_key = MpcState::session_key(
        &payload.session_id.to_string(),
        &payload.user_id.to_string(),
    );

    server_state.sign_sessions.insert(session_key, SignSession {
        tx_message: tx_bytes,
        nonces: None,
        key_package,
        pubkey_package,
        created_at: Instant::now(),
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "signing_session_initialized"
    })))
}

pub async fn sign_round1(
    req: HttpRequest,
    body_bytes: web::Bytes,
    server_state: web::Data<MpcState>,
) -> Result<HttpResponse, MpcError> {
    if !middleware::is_authentic(&req, &body_bytes, &server_state.hmac_secret) {
        return Err(MpcError::Unauthorised("HMAC authentication failed".to_string()));
    }

    let payload: SignRound1Request = serde_json::from_slice(&body_bytes)
        .map_err(|e| MpcError::BadRequest(format!("Invalid JSON: {}", e)))?;

    let session_key = MpcState::session_key(
        &payload.session_id.to_string(),
        &payload.user_id.to_string(),
    );

    let commitments = {
        let mut session = server_state.sign_sessions.get_mut(&session_key)
            .ok_or_else(|| MpcError::NotFound("Signing session not found (run /sign/init first)".to_string()))?;

        let (nonces, commitments) = crypto::frost::sign_round1(&session.key_package);
        session.nonces = Some(nonces);
        commitments
    };

    let commitment_str = serde_json::to_string(&commitments)
        .map_err(|e| MpcError::Internal(format!("Failed to serialize commitments: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "commitment": serde_json::Value::String(commitment_str)
    })))
}

pub async fn sign_round2(
    req: HttpRequest,
    body_bytes: web::Bytes,
    server_state: web::Data<MpcState>,
) -> Result<HttpResponse, MpcError> {
    if !middleware::is_authentic(&req, &body_bytes, &server_state.hmac_secret) {
        return Err(MpcError::Unauthorised("HMAC authentication failed".to_string()));
    }

    let payload: SignRound2Request = serde_json::from_slice(&body_bytes)
        .map_err(|e| MpcError::BadRequest(format!("Invalid JSON: {}", e)))?;

    let session_key = MpcState::session_key(
        &payload.session_id.to_string(),
        &payload.user_id.to_string(),
    );

    let (nonces, key_package, tx_message) = {
        let mut session = server_state.sign_sessions.get_mut(&session_key)
            .ok_or_else(|| MpcError::NotFound("Signing session not found".to_string()))?;

        let n = session.nonces.take()
            .ok_or_else(|| MpcError::BadRequest("Nonces already consumed (run /sign/round1 first)".to_string()))?;

        (n, session.key_package.clone(), session.tx_message.clone())
    };

    let mut commitments_map: BTreeMap<frost::Identifier, frost::round1::SigningCommitments> = BTreeMap::new();

    for (id_str, comm_value) in &payload.data.commitments {
        let identifier = parse_identifier(id_str)?;
        let commitment: frost::round1::SigningCommitments = if let Some(s) = comm_value.as_str() {
            serde_json::from_str(s)
                .map_err(|e| MpcError::BadRequest(format!("Invalid commitment string for {}: {}", id_str, e)))?
        } else {
            serde_json::from_value(comm_value.clone())
                .map_err(|e| MpcError::BadRequest(format!("Invalid commitment value for {}: {}", id_str, e)))?
        };
        commitments_map.insert(identifier, commitment);
    }

    let signing_package = frost::SigningPackage::new(commitments_map, &tx_message);

    let signature_share = crypto::frost::sign_round2(
        &signing_package,
        &nonces,
        &key_package,
    )?;

    let share_bytes = signature_share.serialize();
    let partial_sig_hex = hex::encode(&share_bytes);

    server_state.sign_sessions.remove(&session_key);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "partial_sig": partial_sig_hex
    })))
}
