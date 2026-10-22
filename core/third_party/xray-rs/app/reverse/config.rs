// Module: app\reverse\config.rs
// 1:1 Rust implementation corresponding to Go app\reverse\config.go

use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};

pub use super::config_pb::{BridgeConfig, Control, PortalConfig};

impl Control {
    pub fn fill_in_random(&mut self) {
        let mut rng = rand::thread_rng();
        let len = rng.gen_range(1..=64);
        let mut buf = vec![0u8; len];
        rng.fill_bytes(&mut buf);
        self.random = buf;
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReverseConfig {
    pub bridges: Vec<BridgeConfig>,
    pub portals: Vec<PortalConfig>,
}

impl ReverseConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_bridge(&mut self, tag: impl Into<String>, domain: impl Into<String>) {
        self.bridges.push(BridgeConfig {
            tag: tag.into(),
            domain: domain.into(),
        });
    }

    pub fn add_portal(&mut self, tag: impl Into<String>, domain: impl Into<String>) {
        self.portals.push(PortalConfig {
            tag: tag.into(),
            domain: domain.into(),
        });
    }
}
