use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;
use std::collections::BTreeMap;
use std::time::Instant;
use frost_ed25519 as frost;

use crate::state::{MpcState, DkgSession};
use crate::middleware;
use crate::vault;
use crate::crypto;
use crate::error::MpcError;
use crate::util::parse_identifier;


#[derive(Deserialize)]
pub struct DkgRequest {
    pub session_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Deserialize)]
pub struct DkgRound2Request {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub others: BTreeMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
pub struct DkgFinalizeRequest {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub round2_packages: BTreeMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
pub struct StoreShareRequest {
    pub user_id: Uuid,
    pub share_index: u16,
    pub secret_share: String,
}

pub async fn dkg_round1(
    req: HttpRequest,
    body_bytes: web::Bytes,
    server_state: web::Data<MpcState>,
) -> Result<HttpResponse, MpcError> {
    if !middleware::is_authentic(&req, &body_bytes, &server_state.hmac_secret) {
        return Err(MpcError::Unauthorised("HMAC authentication failed".to_string()));
    }

    let payload: DkgRequest = serde_json::from_slice(&body_bytes)
        .map_err(|e| MpcError::BadRequest(format!("Invalid JSON: {}", e)))?;

    let (round1_secret, round1_package) = crypto::frost::dkg_part1(server_state.node_id)?;

    let session_key = MpcState::session_key(
        &payload.session_id.to_string(),
        &payload.user_id.to_string(),
    );

    server_state.dkg_sessions.insert(session_key, DkgSession {
        round1_secret: Some(round1_secret),
        round2_secret: None,
        received_round1_packages: BTreeMap::new(),
        created_at: Instant::now(),
    });

    let package_json = serde_json::to_value(&round1_package)
        .map_err(|e| MpcError::Internal(format!("Failed to serialize round1 package: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "commitment": package_json
    })))
}

pub async fn dkg_round2(
    req: HttpRequest,
    body_bytes: web::Bytes,
    server_state: web::Data<MpcState>,
) -> Result<HttpResponse, MpcError> {
    if !middleware::is_authentic(&req, &body_bytes, &server_state.hmac_secret) {
        return Err(MpcError::Unauthorised("HMAC authentication failed".to_string()));
    }

    let payload: DkgRound2Request = serde_json::from_slice(&body_bytes)
        .map_err(|e| MpcError::BadRequest(format!("Invalid JSON: {}", e)))?;

    let session_key = MpcState::session_key(
        &payload.session_id.to_string(),
        &payload.user_id.to_string(),
    );

    let round1_secret = {
        let mut session = server_state.dkg_sessions.get_mut(&session_key)
            .ok_or_else(|| MpcError::NotFound("DKG session not found (run round1 first)".to_string()))?;

        session.round1_secret.take()
            .ok_or_else(|| MpcError::BadRequest("Round1 secret already consumed".to_string()))?
    };

    let mut received_round1: BTreeMap<frost::Identifier, frost::keys::dkg::round1::Package> = BTreeMap::new();
    for (id_str, pkg_value) in &payload.others {
        let identifier = parse_identifier(id_str)?;
        let package: frost::keys::dkg::round1::Package = serde_json::from_value(pkg_value.clone())
            .map_err(|e| MpcError::BadRequest(format!("Invalid round1 package for {}: {}", id_str, e)))?;
        received_round1.insert(identifier, package);
    }

    let (round2_secret, round2_packages) = crypto::frost::dkg_part2(
        round1_secret,
        &received_round1,
    )?;

    {
        let mut session = server_state.dkg_sessions.get_mut(&session_key)
            .ok_or_else(|| MpcError::Internal("Session disappeared mid-operation".to_string()))?;
        session.round2_secret = Some(round2_secret);
        session.received_round1_packages = received_round1;
    }

    let mut packages_json = serde_json::Map::new();
    for (identifier, package) in &round2_packages {
        let mut id_key = String::new();
        for (k, _) in &payload.others {
            if let Ok(parsed) = parse_identifier(k) {
                if parsed == *identifier {
                    id_key = k.clone();
                    break;
                }
            }
        }
        if id_key.is_empty() {
            let serialized = serde_json::to_string(identifier).unwrap_or_default();
            id_key = serialized.trim_matches('"').to_string();
        }

        let pkg_str = serde_json::to_string(package)
            .map_err(|e| MpcError::Internal(format!("Failed to serialize round2 package: {}", e)))?;
        packages_json.insert(id_key, serde_json::Value::String(pkg_str));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "round2_packages": packages_json
    })))
}

pub async fn dkg_finalize(
    req: HttpRequest,
    body_bytes: web::Bytes,
    server_state: web::Data<MpcState>,
) -> Result<HttpResponse, MpcError> {
    if !middleware::is_authentic(&req, &body_bytes, &server_state.hmac_secret) {
        return Err(MpcError::Unauthorised("HMAC authentication failed".to_string()));
    }

    let payload: DkgFinalizeRequest = serde_json::from_slice(&body_bytes)
        .map_err(|e| MpcError::BadRequest(format!("Invalid JSON: {}", e)))?;

    let session_key = MpcState::session_key(
        &payload.session_id.to_string(),
        &payload.user_id.to_string(),
    );

    let (round2_secret, received_round1) = {
        let session = server_state.dkg_sessions.get(&session_key)
            .ok_or_else(|| MpcError::NotFound("DKG session not found (run round2 first)".to_string()))?;

        let r2s = session.round2_secret.as_ref()
            .ok_or_else(|| MpcError::BadRequest("Round2 secret not available".to_string()))?
            .clone();

        (r2s, session.received_round1_packages.clone())
    };

    let mut received_round2: BTreeMap<frost::Identifier, frost::keys::dkg::round2::Package> = BTreeMap::new();
    for (id_str, pkg_value) in &payload.round2_packages {
        let identifier = parse_identifier(id_str)?;
        let package: frost::keys::dkg::round2::Package = if let Some(s) = pkg_value.as_str() {
            serde_json::from_str(s)
                .map_err(|e| MpcError::BadRequest(format!("Invalid round2 package string for {}: {}", id_str, e)))?
        } else {
            serde_json::from_value(pkg_value.clone())
                .map_err(|e| MpcError::BadRequest(format!("Invalid round2 package value for {}: {}", id_str, e)))?
        };
        received_round2.insert(identifier, package);
    }

    let (key_package, pubkey_package) = crypto::frost::dkg_part3(
        &round2_secret,
        &received_round1,
        &received_round2,
    )?;

    let key_pkg_json = serde_json::to_vec(&key_package)
        .map_err(|e| MpcError::Internal(format!("Failed to serialize key package: {}", e)))?;
    let pubkey_pkg_json = serde_json::to_vec(&pubkey_package)
        .map_err(|e| MpcError::Internal(format!("Failed to serialize pubkey package: {}", e)))?;

    vault::save_key_package(
        payload.user_id,
        server_state.node_id as i32,
        &key_pkg_json,
        &pubkey_pkg_json,
        &server_state.aes_secret_key,
        &server_state.db_pool,
    ).await?;

    server_state.dkg_sessions.remove(&session_key);

    let verifying_key = pubkey_package.verifying_key();
    let vk_bytes = verifying_key.serialize()
        .map_err(|e| MpcError::Internal(format!("Failed to serialize key: {}", e)))?;
    let public_key_bs58 = bs58::encode(&vk_bytes).into_string();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "public_key": public_key_bs58,
        "status": "key_package_stored"
    })))
}
