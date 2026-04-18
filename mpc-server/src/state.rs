use dashmap::DashMap;
use frost_ed25519 as frost;
use std::collections::BTreeMap;

pub type SessionKey = String;

pub struct DkgSession {
    pub round1_secret: Option<frost::keys::dkg::round1::SecretPackage>,
    pub round2_secret: Option<frost::keys::dkg::round2::SecretPackage>,
    pub received_round1_packages: BTreeMap<frost::Identifier, frost::keys::dkg::round1::Package>,
}

pub struct SignSession {
    pub tx_message: Vec<u8>,
    pub nonces: Option<frost::round1::SigningNonces>,
    pub key_package: frost::keys::KeyPackage,
    pub pubkey_package: frost::keys::PublicKeyPackage,
}

#[derive(Clone)]
pub struct MpcState {
    pub node_id: u16,
    pub hmac_secret: String,
    pub aes_secret_key: String,
    pub dkg_sessions: std::sync::Arc<DashMap<SessionKey, DkgSession>>,
    pub sign_sessions: std::sync::Arc<DashMap<SessionKey, SignSession>>,
}

impl MpcState {
    pub fn new(node_id: u16, hmac_secret: String, aes_secret_key: String) -> Self {
        Self {
            node_id,
            hmac_secret,
            aes_secret_key,
            dkg_sessions: std::sync::Arc::new(DashMap::new()),
            sign_sessions: std::sync::Arc::new(DashMap::new()),
        }
    }

    pub fn session_key(session_id: &str, user_id: &str) -> SessionKey {
        format!("{}:{}", session_id, user_id)
    }
}