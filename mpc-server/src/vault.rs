use uuid::Uuid;
use crate::crypto::aes;
use crate::error::MpcError;
use sqlx::PgPool;

pub async fn save_key_package(
    user_id: Uuid,
    node_id: i32,
    key_package_json: &[u8],
    pubkey_package_json: &[u8],
    aes_master_key_hex: &str,
    pool: &PgPool,
) -> Result<(), MpcError> {
    let enc_key = aes::encrypt(key_package_json, aes_master_key_hex)?;
    let enc_pub = aes::encrypt(pubkey_package_json, aes_master_key_hex)?;

    sqlx::query(
        r#"
        INSERT INTO mpc_vaults (user_id, node_id, key_package, pubkey_package) 
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (user_id, node_id) DO UPDATE 
        SET key_package = EXCLUDED.key_package, pubkey_package = EXCLUDED.pubkey_package
        "#
    )
    .bind(user_id)
    .bind(node_id)
    .bind(enc_key)
    .bind(enc_pub)
    .execute(pool)
    .await
    .map_err(|e| MpcError::Internal(format!("DB insert failed: {}", e)))?;

    Ok(())
}

pub async fn load_key_package(
    user_id: Uuid,
    node_id: i32,
    aes_master_key_hex: &str,
    pool: &PgPool,
) -> Result<(Vec<u8>, Vec<u8>), MpcError> {
    let row: (String, String) = sqlx::query_as(
        "SELECT key_package, pubkey_package FROM mpc_vaults WHERE user_id = $1 AND node_id = $2"
    )
    .bind(user_id)
    .bind(node_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| MpcError::Internal(format!("DB query failed: {}", e)))?
    .ok_or_else(|| MpcError::NotFound(format!("No vault data for user {}", user_id)))?;

    let key_pkg = aes::decrypt(&row.0, aes_master_key_hex)?;
    let pub_pkg = aes::decrypt(&row.1, aes_master_key_hex)?;

    Ok((key_pkg, pub_pkg))
}

pub async fn vault_exists(
    user_id: Uuid,
    node_id: i32,
    pool: &PgPool,
) -> Result<bool, MpcError> {
    let exists: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM mpc_vaults WHERE user_id = $1 AND node_id = $2)"
    )
    .bind(user_id)
    .bind(node_id)
    .fetch_one(pool)
    .await
    .map_err(|e| MpcError::Internal(format!("DB query failed: {}", e)))?;

    Ok(exists.unwrap_or(false))
}