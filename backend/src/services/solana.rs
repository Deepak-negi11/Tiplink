use solana_client::nonblocking::rpc_clinet::RpcClient;
use solana_sdk::{
    message::Message,
    pubkey::Pubkey,
    system_instruction,
    transaction::Transaction,
};
use std::str::FromStr;
use std::error::Error;


pub async fn get_recent_blockhash(
    client:&RpcClient,
)->Result<solana_sdk::hash::Hash , Box<dyn Error>>{

    let blockhash = client.get_latest_blockhash().await?;
    Ok(blockhash)
}


pub async fn build_unsigend_tx(
    client:RpcClient,
    from:&str,
    to:&str,
    amount:u64
)->Result<String, Box<dyn Error>>{

    let sender = Pubkey::from_str(from)?;
    let reciever = Pubkey::from_str(to)?;

    let instruction = system_instruction::transfer(
        &sender,
        &reciever,
        &amount,
    );
    let mut message = Message::new(&[instruction],Some(&sender));
    message.recent_blockhash = get_recent_blockhash(client).await?;
    
    let tx = Transaction::new_unsigned(message);
    let serialzed_bytes = bincode::serialize(&tx)?;
    let base64_payload = base64::encode(serialzed_bytes);

    Ok(base64_payload)
}
pub async fn submit_transaction(
    client: &RpcClient,
    signed_base64_tx: &str,
) -> Result<String, Box<dyn Error>> {
    
    let bytes = base64::decode(signed_base64_tx)?;
    let tx: Transaction = bincode::deserialize(&bytes)?;

    
    if tx.signatures.is_empty() {
        return Err("Transaction is missing signatures. Rejecting.".into());
    }
    let signature = client.send_and_confirm_transaction(&tx).await?;
    
    Ok(signature.to_string())
}