pub mod bridge;
pub mod config;
#[path = "config.pb.rs"]
pub mod config_pb;
pub mod portal;
pub mod reverse;

#[cfg(test)]
pub mod portal_test;

use crate::common::errors::{Error, Result};
use crate::common::net::BoxStream;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

pub use config::{BridgeConfig, PortalConfig, ReverseConfig};

pub struct Portal {
    pub tag: String,
    pub domain: String,
    channel_tx: mpsc::Sender<BoxStream>,
    channel_rx: Arc<Mutex<mpsc::Receiver<BoxStream>>>,
}

impl Portal {
    pub fn new(tag: impl Into<String>, domain: impl Into<String>) -> Self {
        let (tx, rx) = mpsc::channel(128);
        Self {
            tag: tag.into(),
            domain: domain.into(),
            channel_tx: tx,
            channel_rx: Arc::new(Mutex::new(rx)),
        }
    }

    pub async fn dispatch(&self, stream: BoxStream) -> Result<()> {
        self.channel_tx
            .send(stream)
            .await
            .map_err(|_| Error::Closed)
    }

    pub async fn pull_stream(&self) -> Option<BoxStream> {
        let mut rx = self.channel_rx.lock().await;
        rx.recv().await
    }
}

pub struct Bridge {
    pub tag: String,
    pub domain: String,
}

impl Bridge {
    pub fn new(tag: impl Into<String>, domain: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            domain: domain.into(),
        }
    }
}

pub struct ReverseManager {
    portals: HashMap<String, Arc<Portal>>,
    bridges: HashMap<String, Arc<Bridge>>,
}

impl Default for ReverseManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ReverseManager {
    pub fn new() -> Self {
        Self {
            portals: HashMap::new(),
            bridges: HashMap::new(),
        }
    }

    pub fn with_config(config: ReverseConfig) -> Self {
        let mut portals = HashMap::new();
        for p in config.portals {
            portals.insert(p.tag.clone(), Arc::new(Portal::new(p.tag, p.domain)));
        }

        let mut bridges = HashMap::new();
        for b in config.bridges {
            bridges.insert(b.tag.clone(), Arc::new(Bridge::new(b.tag, b.domain)));
        }

        Self { portals, bridges }
    }

    pub fn register_portal(&mut self, portal: Portal) {
        self.portals.insert(portal.tag.clone(), Arc::new(portal));
    }

    pub fn register_bridge(&mut self, bridge: Bridge) {
        self.bridges.insert(bridge.tag.clone(), Arc::new(bridge));
    }

    pub fn get_portal(&self, key: &str) -> Option<Arc<Portal>> {
        if let Some(p) = self.portals.get(key) {
            return Some(p.clone());
        }
        for p in self.portals.values() {
            if p.domain == key {
                return Some(p.clone());
            }
        }
        None
    }

    pub fn get_bridge(&self, key: &str) -> Option<Arc<Bridge>> {
        if let Some(b) = self.bridges.get(key) {
            return Some(b.clone());
        }
        for b in self.bridges.values() {
            if b.domain == key {
                return Some(b.clone());
            }
        }
        None
    }
}
