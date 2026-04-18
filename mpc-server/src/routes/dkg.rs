use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;
use std::collections::BTreeMap;
use frost_ed25519 as frost;

use crate::state::{MpcState, DkgSession};
use crate::middleware;
use crate::vault;
use crate::crypto;
use crate::error::MpcError;

fn parse_identifier(id_str: &str) -> Result<frost::Identifier, MpcError> {
    if let Ok(num) = id_str.parse::<u16>() {
        return num.try_into()
            .map_err(|_| MpcError::BadRequest(format!("Invalid FROST identifier from u16: {}", id_str)));
    }
    let quoted = format!("\"{}\"", id_str);
    serde_json::from_str::<frost::Identifier>(&quoted)
        .map_err(|e| MpcError::BadRequest(format!("Cannot parse FROST identifier '{}': {}", id_str, e)))
}

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
        let id_key = serde_json::to_string(identifier)
            .map_err(|e| MpcError::Internal(format!("Failed to serialize identifier: {}", e)))?;
        let id_key = id_key.trim_matches('"').to_string();
        let pkg_value = serde_json::to_value(package)
            .map_err(|e| MpcError::Internal(format!("Failed to serialize round2 package: {}", e)))?;
        packages_json.insert(id_key, pkg_value);
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
        let package: frost::keys::dkg::round2::Package = serde_json::from_value(pkg_value.clone())
            .map_err(|e| MpcError::BadRequest(format!("Invalid round2 package for {}: {}", id_str, e)))?;
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
        &key_pkg_json,
        &pubkey_pkg_json,
        &server_state.aes_secret_key,
    )?;

    server_state.dkg_sessions.remove(&session_key);

    let verifying_key = pubkey_package.verifying_key();
    let vk_bytes = verifying_key.serialize();
    let public_key_hex = hex::encode(&vk_bytes);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "public_key": public_key_hex,
        "status": "key_package_stored"
    })))
}

pub async fn store_share(
    req: HttpRequest,
    body_bytes: web::Bytes,
    server_state: web::Data<MpcState>,
) -> Result<HttpResponse, MpcError> {
    if !middleware::is_authentic(&req, &body_bytes, &server_state.hmac_secret) {
        return Err(MpcError::Unauthorised("HMAC authentication failed".to_string()));
    }

    let payload: StoreShareRequest = serde_json::from_slice(&body_bytes)
        .map_err(|e| MpcError::BadRequest(format!("Invalid JSON: {}", e)))?;

    if payload.share_index != server_state.node_id {
        return Err(MpcError::BadRequest("Wrong node ID for this share".to_string()));
    }

    vault::save_encrypted(
        payload.user_id,
        "raw_share",
        payload.secret_share.as_bytes(),
        &server_state.aes_secret_key,
    )?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "Shard safely locked in the Vault."
    })))
}