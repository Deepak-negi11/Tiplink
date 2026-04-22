use std::collections::HashMap;
use uuid::Uuid;
use std::sync::{Arc, RwLock};

use crate::db::pool::DbPool;
use crate::db::queries;

#[derive(Clone)]
pub struct TrackedAccounts {
    inner: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl TrackedAccounts {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn refresh_from_db(&self, pool: &DbPool) {
        let addresses = queries::load_tracked_addresses(pool);
        let mut map = self.inner.write().expect("Lock poisoned");
        map.clear();
        for addr in &addresses {
            map.insert(addr.public_key.clone(), addr.id);
        }
        tracing::info!("Tracking {} wallet addresses", map.len());
    }

    pub fn get_user_id(&self, address: &str) -> Option<Uuid> {
        let map = self.inner.read().expect("Lock poisoned");
        map.get(address).copied()
    }

    pub fn is_tracked(&self, address: &str) -> bool {
        let map = self.inner.read().expect("Lock poisoned");
        map.contains_key(address)
    }

    pub fn all_addresses(&self) -> Vec<String> {
        let map = self.inner.read().expect("Lock poisoned");
        map.keys().cloned().collect()
    }
}
