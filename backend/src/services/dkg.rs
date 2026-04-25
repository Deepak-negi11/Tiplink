use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;
use std::collections::BTreeMap;
use crate::error::AppError;
use crate::services::hmac::post_to_node;

pub struct Config {
    pub aws: String,
    pub do_ocean: String,
    pub cloudflare: String,
    pub api_keys: String,
}

impl Config {
    pub fn node_urls(&self) -> Vec<(&str, u16)> {
        vec![
            (&self.aws, 1),
            (&self.do_ocean, 2),
            (&self.cloudflare, 3),
        ]
    }
}

pub async fn generate_keypair(config: &Config, user_id: Uuid) -> Result<String, AppError> {
    let client = Client::new();
    let session_id = Uuid::new_v4();
    let key = &config.api_keys;

    let body1 = json!({ "session_id": session_id, "user_id": user_id }).to_string();

    let (r1_aws, r1_do, r1_cf) = tokio::try_join!(
        post_to_node(&client, &config.aws, "/dkg/round1", &body1, key),
        post_to_node(&client, &config.do_ocean, "/dkg/round1", &body1, key),
        post_to_node(&client, &config.cloudflare, "/dkg/round1", &body1, key),
    )?;

    let c1 = &r1_aws["commitment"];
    let c2 = &r1_do["commitment"];
    let c3 = &r1_cf["commitment"];

    let body2_aws = json!({
        "session_id": session_id, "user_id": user_id,
        "others": { "2": c2, "3": c3 }
    }).to_string();
    let body2_do = json!({
        "session_id": session_id, "user_id": user_id,
        "others": { "1": c1, "3": c3 }
    }).to_string();
    let body2_cf = json!({
        "session_id": session_id, "user_id": user_id,
        "others": { "1": c1, "2": c2 }
    }).to_string();

    let (r2_aws, r2_do, r2_cf) = tokio::try_join!(
        post_to_node(&client, &config.aws, "/dkg/round2", &body2_aws, key),
        post_to_node(&client, &config.do_ocean, "/dkg/round2", &body2_do, key),
        post_to_node(&client, &config.cloudflare, "/dkg/round2", &body2_cf, key),
    )?;

    let r2_pkgs_aws = &r2_aws["round2_packages"];
    let r2_pkgs_do = &r2_do["round2_packages"];
    let r2_pkgs_cf = &r2_cf["round2_packages"];

    println!("r2_pkgs_aws keys: {:?}", r2_pkgs_aws);
    println!("r2_pkgs_do keys: {:?}", r2_pkgs_do);
    println!("r2_pkgs_cf keys: {:?}", r2_pkgs_cf);

    let body3_aws = json!({
        "session_id": session_id, "user_id": user_id,
        "round2_packages": {
            "2": r2_pkgs_do.get("1").unwrap_or(&Value::Null),
            "3": r2_pkgs_cf.get("1").unwrap_or(&Value::Null),
        }
    }).to_string();
    let body3_do = json!({
        "session_id": session_id, "user_id": user_id,
        "round2_packages": {
            "1": r2_pkgs_aws.get("2").unwrap_or(&Value::Null),
            "3": r2_pkgs_cf.get("2").unwrap_or(&Value::Null),
        }
    }).to_string();
    let body3_cf = json!({
        "session_id": session_id, "user_id": user_id,
        "round2_packages": {
            "1": r2_pkgs_aws.get("3").unwrap_or(&Value::Null),
            "2": r2_pkgs_do.get("3").unwrap_or(&Value::Null),
        }
    }).to_string();

    println!("body3_aws: {}", body3_aws);
    println!("body3_do: {}", body3_do);
    println!("body3_cf: {}", body3_cf);

    let (r3_aws, _r3_do, _r3_cf) = tokio::try_join!(
        post_to_node(&client, &config.aws, "/dkg/finalize", &body3_aws, key),
        post_to_node(&client, &config.do_ocean, "/dkg/finalize", &body3_do, key),
        post_to_node(&client, &config.cloudflare, "/dkg/finalize", &body3_cf, key),
    )?;

    let public_key = r3_aws["public_key"]
        .as_str()
        .ok_or_else(|| AppError::ExternalApi("Missing public_key from DKG finalize".into()))?
        .to_string();

    Ok(public_key)
}
