use dashmap::DashMap;
use frost_ed25519 as frost;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

pub type SessionKey = String;

const SESSION_TTL_SECS: u64 = 300;

pub struct DkgSession {
    pub round1_secret: Option<frost::keys::dkg::round1::SecretPackage>,
    pub round2_secret: Option<frost::keys::dkg::round2::SecretPackage>,
    pub received_round1_packages: BTreeMap<frost::Identifier, frost::keys::dkg::round1::Package>,
    pub created_at: Instant,
}

pub struct SignSession {
    pub tx_message: Vec<u8>,
    pub nonces: Option<frost::round1::SigningNonces>,
    pub key_package: frost::keys::KeyPackage,
    pub pubkey_package: frost::keys::PublicKeyPackage,
    pub created_at: Instant,
}

#[derive(Clone)]
pub struct MpcState {
    pub node_id: u16,
    pub hmac_secret: String,
    pub aes_secret_key: String,
    pub db_pool: sqlx::PgPool,
    pub dkg_sessions: Arc<DashMap<SessionKey, DkgSession>>,
    pub sign_sessions: Arc<DashMap<SessionKey, SignSession>>,
}

impl MpcState {
    pub fn new(node_id: u16, hmac_secret: String, aes_secret_key: String, db_pool: sqlx::PgPool) -> Self {
        Self {
            node_id,
            hmac_secret,
            aes_secret_key,
            db_pool,
            dkg_sessions: Arc::new(DashMap::new()),
            sign_sessions: Arc::new(DashMap::new()),
        }
    }

    pub fn session_key(session_id: &str, user_id: &str) -> SessionKey {
        format!("{}:{}", session_id, user_id)
    }

    pub fn purge_expired_sessions(&self) {
        let now = Instant::now();

        self.dkg_sessions.retain(|_key, session| {
            now.duration_since(session.created_at).as_secs() < SESSION_TTL_SECS
        });

        self.sign_sessions.retain(|_key, session| {
            now.duration_since(session.created_at).as_secs() < SESSION_TTL_SECS
        });
    }
}