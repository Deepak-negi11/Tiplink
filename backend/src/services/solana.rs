use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    hash::Hash,
    message::Message,
    pubkey::Pubkey,
    signature::Signature,
    system_instruction,
    transaction::Transaction
};
use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account
};
use spl_token::instruction::transfer as spl_transfer;
use std::str::FromStr;
use crate::error::AppError;

pub async fn get_recent_blockhash(rpc_url: &str) -> Result<Hash, AppError> {
    let client = RpcClient::new(rpc_url.to_string());
    client.get_latest_blockhash().await
        .map_err(|_| AppError::ExternalApi("Failed to fetch recent blockhash".to_string()))
}

pub async fn build_transfer_tx(
    rpc_url: &str,
    from_pubkey: &str,
    to_pubkey: &str,
    amount_lamports: u64,
) -> Result<Vec<u8>, AppError> {
    let client = RpcClient::new(rpc_url.to_string());
    let sender = Pubkey::from_str(from_pubkey)
        .map_err(|_| AppError::BadRequest("Invalid sender pubkey".to_string()))?;
    let receiver = Pubkey::from_str(to_pubkey)
        .map_err(|_| AppError::BadRequest("Invalid receiver pubkey".to_string()))?;

    let instruction = system_instruction::transfer(
        &sender,
        &receiver,
        amount_lamports
    );
    let recent_blockhash = client.get_latest_blockhash().await
        .map_err(|_| AppError::ExternalApi("Failed to fetch blockhash".to_string()))?;

    let message = Message::new(&[instruction], Some(&sender));

    let mut tx = Transaction::new_unsigned(message);
    tx.message.recent_blockhash = recent_blockhash;
    bincode::serialize(&tx)
        .map_err(|_| AppError::InternalServerError("Failed to serialize transaction".to_string()))
}

pub async fn build_spl_transfer_tx(
    rpc_url: &str,
    from_pubkey: &str,
    to_pubkey: &str,
    mint_pubkey: &str,
    amount: u64
) -> Result<Vec<u8>, AppError> {
    let client = RpcClient::new(rpc_url.to_string());
    let sender = Pubkey::from_str(from_pubkey)
        .map_err(|_| AppError::BadRequest("Invalid sender pubkey".to_string()))?;
    let receiver = Pubkey::from_str(to_pubkey)
        .map_err(|_| AppError::BadRequest("Invalid receiver pubkey".to_string()))?;
    let mint = Pubkey::from_str(mint_pubkey)
        .map_err(|_| AppError::BadRequest("Invalid mint pubkey".to_string()))?;

    let from_ata = get_associated_token_address(&sender, &mint);
    let to_ata = get_associated_token_address(&receiver, &mint);
    
    let mut instructions = Vec::new();

    // Check if the recipient ATA exists on chain natively
    if client.get_account(&to_ata).await.is_err() {
        // Assume it doesn't exist, execute instruction for ATA spawn
        instructions.push(create_associated_token_account(
            &sender,
            &receiver,
            &mint,
            &spl_token::id()
        ));
    }

    instructions.push(spl_transfer(
        &spl_token::id(),
        &from_ata,
        &to_ata,
        &sender, // owner
        &[],
        amount
    ).map_err(|_| AppError::InternalServerError("Failed to build SPL transfer instruction".to_string()))?);

    let recent_blockhash = client.get_latest_blockhash().await
        .map_err(|_| AppError::ExternalApi("Failed to fetch blockhash".to_string()))?;

    let message = Message::new(&instructions, Some(&sender));
    let mut tx = Transaction::new_unsigned(message);
    tx.message.recent_blockhash = recent_blockhash;
    
    bincode::serialize(&tx)
        .map_err(|_| AppError::InternalServerError("Failed to serialize SPL transaction".to_string()))
}

pub async fn submit_transaction(
    rpc_url: &str,
    signed_tx_bytes: &[u8],
) -> Result<String, AppError> {
    let client = RpcClient::new(rpc_url.to_string());

    let tx: Transaction = bincode::deserialize(signed_tx_bytes)
        .map_err(|_| AppError::BadRequest("Failed to deserialize signed transaction".to_string()))?;

    if tx.signatures.is_empty() {
        return Err(AppError::BadRequest("Transaction is missing signature".to_string()));
    }
    
    let signature: Signature = client.send_transaction(&tx).await
        .map_err(|_| AppError::ExternalApi("RPC rejected the transaction".to_string()))?;
        
    Ok(signature.to_string())
}

pub fn extract_destination(tx_bytes: &[u8]) -> Result<String, AppError> {
    let tx: Transaction = bincode::deserialize(tx_bytes)
        .map_err(|_| AppError::BadRequest("Failed to decrypt transaction target".to_string()))?;

    let account_keys = &tx.message.account_keys;
    for instruction in &tx.message.instructions {
        if instruction.program_id_index < account_keys.len() as u8 {
            let program_id = account_keys[instruction.program_id_index as usize];
            if program_id == solana_sdk::system_program::id() {
                if instruction.accounts.len() >= 2 {
                    let to_index = instruction.accounts[1];
                    return Ok(account_keys[to_index as usize].to_string());
                }
            } else if program_id == spl_token::id() {
                if instruction.accounts.len() >= 2 {
                    let to_index = instruction.accounts[1];
                    return Ok(account_keys[to_index as usize].to_string());
                }
            }
        }
    }
    Err(AppError::BadRequest("No valid transfer instruction found".to_string()))
}

pub fn extract_amount(tx_bytes: &[u8]) -> Result<u64, AppError> {
    let tx: Transaction = bincode::deserialize(tx_bytes)
        .map_err(|_| AppError::BadRequest("Failed to decrypt transaction instruction".to_string()))?;

    let account_keys = &tx.message.account_keys;
    for instruction in &tx.message.instructions {
        if instruction.program_id_index < account_keys.len() as u8 {
            let program_id = account_keys[instruction.program_id_index as usize];
            if program_id == solana_sdk::system_program::id() {
                if instruction.data.len() >= 12 {
                    let mut arr = [0u8; 8];
                    arr.copy_from_slice(&instruction.data[4..12]);
                    return Ok(u64::from_le_bytes(arr));
                }
            } else if program_id == spl_token::id() {
                if instruction.data.len() >= 9 {
                    let mut arr = [0u8; 8];
                    arr.copy_from_slice(&instruction.data[1..9]);
                    return Ok(u64::from_le_bytes(arr));
                }
            }
        }
    }
    Err(AppError::BadRequest("Unable to map underlying asset amounts".to_string()))
}