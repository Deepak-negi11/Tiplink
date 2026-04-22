use reqwest::Client;
use serde_json::Value;
use uuid::Uuid;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use chrono::Utc;

use crate::error::AppError;

pub fn hmac_sign(body: &str, api_key: &str) -> (String, String) {
    let timestamp = Utc::now().timestamp().to_string();
    let mut mac = Hmac::<Sha256>::new_from_slice(api_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(body.as_bytes());
    mac.update(timestamp.as_bytes());
    let result = mac.finalize();
    (format!("{:x}", result.into_bytes()), timestamp)
}

pub async fn post_to_node(
    client: &Client,
    url: &str,
    endpoint: &str,
    body_str: &str,
    api_key: &str,
) -> Result<Value, AppError> {
    let (signature, timestamp) = hmac_sign(body_str, api_key);

    let res = client.post(format!("{}{}", url, endpoint))
        .header("X-Signature", signature)
        .header("X-Timestamp", timestamp)
        .header("Content-Type", "application/json")
        .body(body_str.to_string())
        .send().await
        .map_err(|e| AppError::ExternalApi(format!("Node connection failed: {}", e)))?
        .json::<Value>().await
        .map_err(|e| AppError::ExternalApi(format!("Invalid JSON from node: {}", e)))?;

    Ok(res)
}

pub async fn post_to_node_with_session(
    client: &Client,
    url: &str,
    endpoint: &str,
    session: Uuid,
    user: Uuid,
    payload: Value,
    api_key: &str,
) -> Result<Value, AppError> {
    let body_str = serde_json::json!({
        "session_id": session,
        "user_id": user,
        "data": payload
    }).to_string();

    post_to_node(client, url, endpoint, &body_str, api_key).await
}
