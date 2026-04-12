use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use crate::error::AppError;

pub struct Config {
    pub aws: String,
    pub do_ocean: String,
    pub cloudflare: String,
    pub api_keys: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DkgRound1Data {
    pub commitment: String,
}

pub fn hmac_sign(body: &str, api_key: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(api_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(body.as_bytes());
    let result = mac.finalize();
    format!("{:x}", result.into_bytes())
}

pub async fn dkg_round1(url: &str, session_id: Uuid, user_id: Uuid, api_key: &str, client: &Client) -> Result<DkgRound1Data, AppError> {
    let body_str = json!({ "session_id": session_id, "user_id": user_id }).to_string();
    let signature = hmac_sign(&body_str, api_key);
    
    let res = client.post(format!("{}/dkg/round1", url))
        .header("X-Signature", signature)
        .header("Content-Type", "application/json")
        .body(body_str)
        .send().await
        .map_err(|_| AppError::ExternalApi("DKG Round 1 connection failed".to_string()))?
        .json::<Value>().await
        .map_err(|_| AppError::ExternalApi("DKG invalid JSON payload".to_string()))?;
        
    let comm = res["commitment"].as_str().ok_or_else(|| AppError::ExternalApi("Missing commitment".into()))?.to_string();
    Ok(DkgRound1Data { commitment: comm })
}

pub async fn dkg_round2(url: &str, session_id: Uuid, user_id: Uuid, others: Vec<DkgRound1Data>, api_key: &str, client: &Client) -> Result<String, AppError> {
    let body_str = json!({ 
        "session_id": session_id, 
        "user_id": user_id, 
        "others": others 
    }).to_string();
    let signature = hmac_sign(&body_str, api_key);
    
    let res = client.post(format!("{}/dkg/round2", url))
        .header("X-Signature", signature)
        .header("Content-Type", "application/json")
        .body(body_str)
        .send().await
        .map_err(|_| AppError::ExternalApi("DKG Round 2 connection failed".to_string()))?
        .json::<Value>().await
        .map_err(|_| AppError::ExternalApi("DKG invalid JSON payload".to_string()))?;
        
    let pubkey = res["public_key"].as_str().ok_or_else(|| AppError::ExternalApi("Missing public_key".into()))?.to_string();
    Ok(pubkey)
}

pub async fn generate_keypair(config: &Config, user_id: Uuid) -> Result<String, AppError> {
    let client = Client::new();
    let session_id = Uuid::new_v4();
    let key = &config.api_keys;

    // ROUND 1: Simultaneous POST
    let (r1_aws, r1_do, r1_cf) = tokio::try_join!(
        dkg_round1(&config.aws, session_id, user_id, key, &client),
        dkg_round1(&config.do_ocean, session_id, user_id, key, &client),
        dkg_round1(&config.cloudflare, session_id, user_id, key, &client)
    )?;

    // ROUND 2: Forwarding generated permutations
    let aws_others = vec![DkgRound1Data{commitment: r1_do.commitment.clone()}, DkgRound1Data{commitment: r1_cf.commitment.clone()}];
    let do_others = vec![DkgRound1Data{commitment: r1_aws.commitment.clone()}, DkgRound1Data{commitment: r1_cf.commitment.clone()}];
    let cf_others = vec![DkgRound1Data{commitment: r1_aws.commitment.clone()}, DkgRound1Data{commitment: r1_do.commitment.clone()}];

    let (r2_aws, _r2_do, _r2_cf) = tokio::try_join!(
        dkg_round2(&config.aws, session_id, user_id, aws_others, key, &client),
        dkg_round2(&config.do_ocean, session_id, user_id, do_others, key, &client),
        dkg_round2(&config.cloudflare, session_id, user_id, cf_others, key, &client)
    )?;

    // All servers mathematically derive the exact same public key
    Ok(r2_aws)
}
