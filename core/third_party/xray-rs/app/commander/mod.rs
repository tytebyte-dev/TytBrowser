pub mod commander;
#[path = "config.pb.rs"]
pub mod config_pb;
pub mod outbound;
pub mod service;

#[cfg(test)]
pub mod commander_test;

use crate::common::errors::{Error, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use config_pb::{Config as CommanderConfig, ReflectionConfig};
pub use outbound::CommanderOutbound;

pub trait Service: Send + Sync {
    fn service_name(&self) -> &str;
}

pub struct Commander {
    pub tag: String,
    pub listen: String,
    services: Arc<RwLock<HashMap<String, Arc<dyn Service>>>>,
}

impl Commander {
    pub fn new(tag: String, listen: String) -> Self {
        Self {
            tag,
            listen,
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_service(&self, service: Arc<dyn Service>) {
        let mut guard = self.services.write().await;
        guard.insert(service.service_name().to_string(), service);
    }

    pub async fn get_service(&self, name: &str) -> Result<Arc<dyn Service>> {
        let guard = self.services.read().await;
        guard
            .get(name)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Service {} not found in Commander", name)))
    }

    pub async fn service_count(&self) -> usize {
        self.services.read().await.len()
    }
}
