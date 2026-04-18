use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key,
};
use crate::error::MpcError;

pub fn encrypt(plaintext: &[u8], aes_master_key_hex: &str) -> Result<String, MpcError> {
    let key_bytes = hex::decode(aes_master_key_hex)
        .map_err(|_| MpcError::Crypto("Failed to decode AES master key hex".to_string()))?;

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher.encrypt(&nonce, plaintext)
        .map_err(|_| MpcError::Crypto("AES-256-GCM encryption failed".to_string()))?;

    let mut payload = nonce.to_vec();
    payload.extend_from_slice(&ciphertext);

    Ok(hex::encode(payload))
}

pub fn decrypt(encrypted_hex: &str, aes_master_key_hex: &str) -> Result<Vec<u8>, MpcError> {
    let key_bytes = hex::decode(aes_master_key_hex)
        .map_err(|_| MpcError::Crypto("Failed to decode AES master key hex".to_string()))?;

    let payload_bytes = hex::decode(encrypted_hex)
        .map_err(|_| MpcError::Crypto("Failed to decode encrypted payload hex".to_string()))?;

    if payload_bytes.len() < 12 {
        return Err(MpcError::Crypto("Encrypted payload too short (nonce missing)".to_string()));
    }

    let (nonce_bytes, ciphertext) = payload_bytes.split_at(12);
    let nonce = aes_gcm::Nonce::from_slice(nonce_bytes);

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|_| MpcError::Crypto("AES-256-GCM decryption failed (wrong key or corrupted data)".to_string()))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key_hex = "75e166cd1d856605e8c55cd35d076e71b023e24036fcb53753edd0da8be7159c";
        let plaintext = b"secret key share data";

        let encrypted = encrypt(plaintext, key_hex).unwrap();
        let decrypted = decrypt(&encrypted, key_hex).unwrap();

        assert_eq!(decrypted, plaintext);
    }
}
