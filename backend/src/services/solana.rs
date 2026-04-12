use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    hash::Hash,
    message::Message,
    pubkey::Pubkey,
    signature::Signature,
    system_instruction,
    transaction::Transaction
};
use std::str::FromStr;
use std::error::Error;

pub async fn get_recent_blockhash(rpc_url:&str) -> Result<Hash, Box<dyn Error>>{
    let client = RpcClient::new(rpc_url.to_string());
    let latest_blockhash = client.get_latest_blockhash().await?;
    Ok(latest_blockhash)
}

pub async fn build_transfer_tx(
    rpc_url:&str,
    from_pubkey:&str,
    to_pubkey:&str,
    amount_lamports:u64,
)-> Result<Vec<u8> , Box<dyn Error>>{
    let client = RpcClient::new(rpc_url.to_string());
    let sender = Pubkey::from_str(from_pubkey)?;
    let reciever = Pubkey::from_str(to_pubkey)?;

    let instruction = system_instruction::transfer(
        &sender,
        &reciever,
        amount_lamports
    );
    let recent_blockhash = client.get_latest_blockhash().await?;

    let message = Message::new(&[instruction],Some(&sender));

    let mut tx = Transaction::new_unsigned(message);
    tx.message.recent_blockhash = recent_blockhash;
    let tx_bytes = bincode::serialize(&tx)?;
    Ok(tx_bytes)
}

pub async fn submit_transaction(
    rpc_url:&str,
    signed_tx_bytes:&[u8],
)->Result<String, Box<dyn Error>>{
    let client = RpcClient::new(rpc_url.to_string());

    let tx:Transaction = bincode::deserialize(signed_tx_bytes)?;

    if tx.signatures.is_empty(){
        return Err("Transaction is missing signature".into());
    }
    let signature: Signature = client.send_transaction(&tx).await?;
    Ok(signature.to_string())
}