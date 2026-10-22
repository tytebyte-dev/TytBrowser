// Module: app\policy\policy.rs
// 1:1 Rust implementation corresponding to Go app\policy\policy.go

use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionPolicy {
    pub handshake: Duration,
    pub conn_idle: Duration,
    pub uplink_only: Duration,
    pub downlink_only: Duration,
    pub buffer_size: usize,
    pub stats_user_uplink: bool,
    pub stats_user_downlink: bool,
}

impl Default for SessionPolicy {
    fn default() -> Self {
        Self {
            handshake: Duration::from_secs(4),
            conn_idle: Duration::from_secs(300),
            uplink_only: Duration::from_secs(2),
            downlink_only: Duration::from_secs(5),
            buffer_size: 512 * 1024,
            stats_user_uplink: false,
            stats_user_downlink: false,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SystemPolicy {
    pub stats_inbound_uplink: bool,
    pub stats_inbound_downlink: bool,
    pub stats_outbound_uplink: bool,
    pub stats_outbound_downlink: bool,
}

#[derive(Default)]
pub struct PolicyManager {
    levels: HashMap<u32, SessionPolicy>,
    system: SystemPolicy,
}

impl PolicyManager {
    pub fn new(levels: HashMap<u32, SessionPolicy>, system: SystemPolicy) -> Self {
        Self { levels, system }
    }

    pub fn for_level(&self, level: u32) -> SessionPolicy {
        self.levels.get(&level).cloned().unwrap_or_default()
    }

    pub fn for_system(&self) -> &SystemPolicy {
        &self.system
    }

    pub fn set_level(&mut self, level: u32, policy: SessionPolicy) {
        self.levels.insert(level, policy);
    }
}
