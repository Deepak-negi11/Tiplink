use actix_web::{web, HttpResponse, HttpRequest};
use serde::Deserialize;
use uuid::Uuid;
use frost_ed25519 as frost;

use crate::state::MpcState;
use crate::vault;
use crate::error::MpcError;

#[derive(Deserialize)]
pub struct PubkeyRequest {
    pub user_id: Uuid,
}

/// Retrieves the stored PublicKeyPackage for a user from this node's vault.
/// Called by the backend during FROST signature aggregation.
pub async fn get_pubkey_package(
    state: web::Data<MpcState>,
    body: web::Json<PubkeyRequest>,
) -> Result<HttpResponse, MpcError> {
    let user_id = body.user_id;

    let (_key_pkg_bytes, pubkey_pkg_bytes) = vault::load_key_package(
        user_id,
        state.node_id as i32,
        &state.aes_secret_key,
        &state.db_pool,
    ).await?;

    let pubkey_package: frost::keys::PublicKeyPackage = serde_json::from_slice(&pubkey_pkg_bytes)
        .map_err(|e| MpcError::Internal(format!("Failed to deserialize PublicKeyPackage: {}", e)))?;

    let pubkey_package_str = serde_json::to_string(&pubkey_package)
        .map_err(|e| MpcError::Internal(format!("Failed to serialize pubkey_package: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "pubkey_package": serde_json::Value::String(pubkey_package_str),
        "user_id": user_id,
    })))
}
