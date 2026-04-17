use aes_gcm::{
    aead::{Aead,AeadCore,KeyInit,OsRng},
    Aes256Gcm,Key
};
use std::fs;
use std::path::Path;
use uuid::Uuid;

pub fn encrypt_and_save(
    user_id:Uuid,
    raw_secret_share:&str,
    aes_master_key_hex:&str
)-> Result<()>{
    let key_bytes = hex::decode(aes_master_key_hex)
        .map_err(|_| "Failed to decode the Master Aes Key".to_string())?;

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher.encrypt(&nonce, raw_secret_share.as_bytes())
        .map_err(|_| "Encryption completely failed!" .to_string())?;


    let mut final_payload = nonce.to_vec();
    final_payload.extend_from_slice(&ciphertext);

    let safe_string = hex::encode(final_payload);

    let file_path = format!("./vaults/{}.txt",user_id);
    fs::write(Path::new(&file_path), safe_string)
        .map_err(|_| "Failed to write the file to the hard drive".to_string())?;

    Ok(())
}