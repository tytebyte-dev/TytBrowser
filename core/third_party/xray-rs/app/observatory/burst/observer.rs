// Module: app\observatory\burst\observer.rs
// 1:1 Rust implementation corresponding to Go app\observatory\burst\observer.go

use super::health::HealthStatus;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct BurstObserver {
    statuses: Arc<RwLock<HashMap<String, HealthStatus>>>,
}

impl BurstObserver {
    pub fn new() -> Self {
        Self {
            statuses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn update(&self, status: HealthStatus) {
        let mut guard = self.statuses.write().await;
        guard.insert(status.tag.clone(), status);
    }

    pub async fn get(&self, tag: &str) -> Option<HealthStatus> {
        let guard = self.statuses.read().await;
        guard.get(tag).cloned()
    }
}

impl Default for BurstObserver {
    fn default() -> Self {
        Self::new()
    }
}
