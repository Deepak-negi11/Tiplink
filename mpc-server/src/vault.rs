use std::fs;
use std::path::Path;
use uuid::Uuid;
use crate::crypto::aes;
use crate::error::MpcError;

const VAULT_DIR: &str = "./vaults";

fn ensure_vault_dir() -> Result<(), MpcError> {
    fs::create_dir_all(VAULT_DIR)
        .map_err(|e| MpcError::Internal(format!("Failed to create vault directory: {}", e)))
}

fn vault_path(user_id: Uuid, suffix: &str) -> String {
    format!("{}/{}_{}.vault", VAULT_DIR, user_id, suffix)
}

pub fn save_encrypted(
    user_id: Uuid,
    label: &str,
    data: &[u8],
    aes_master_key_hex: &str,
) -> Result<(), MpcError> {
    ensure_vault_dir()?;
    let encrypted = aes::encrypt(data, aes_master_key_hex)?;
    let file_path = vault_path(user_id, label);
    fs::write(Path::new(&file_path), encrypted)
        .map_err(|e| MpcError::Internal(format!("Failed to write vault file: {}", e)))?;
    Ok(())
}

pub fn load_decrypted(
    user_id: Uuid,
    label: &str,
    aes_master_key_hex: &str,
) -> Result<Vec<u8>, MpcError> {
    let file_path = vault_path(user_id, label);
    let encrypted_hex = fs::read_to_string(Path::new(&file_path))
        .map_err(|_| MpcError::NotFound(format!("No vault data for user {} ({})", user_id, label)))?;
    aes::decrypt(&encrypted_hex, aes_master_key_hex)
}

pub fn vault_exists(user_id: Uuid, label: &str) -> bool {
    Path::new(&vault_path(user_id, label)).exists()
}

pub fn save_key_package(
    user_id: Uuid,
    key_package_json: &[u8],
    pubkey_package_json: &[u8],
    aes_master_key_hex: &str,
) -> Result<(), MpcError> {
    save_encrypted(user_id, "key_package", key_package_json, aes_master_key_hex)?;
    save_encrypted(user_id, "pubkey_package", pubkey_package_json, aes_master_key_hex)?;
    Ok(())
}

pub fn load_key_package(
    user_id: Uuid,
    aes_master_key_hex: &str,
) -> Result<(Vec<u8>, Vec<u8>), MpcError> {
    let key_pkg = load_decrypted(user_id, "key_package", aes_master_key_hex)?;
    let pubkey_pkg = load_decrypted(user_id, "pubkey_package", aes_master_key_hex)?;
    Ok((key_pkg, pubkey_pkg))
}