// Module: app\policy\config.pb.rs
// 1:1 Rust protobuf message definitions corresponding to Go app\policy\config.pb.go

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Second {
    #[serde(default)]
    pub value: u32,
}

impl Second {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

/// Timeout is a message for timeout settings in various stages, in seconds.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyTimeout {
    #[serde(default)]
    pub handshake: Option<Second>,
    #[serde(default)]
    pub connection_idle: Option<Second>,
    #[serde(default)]
    pub uplink_only: Option<Second>,
    #[serde(default)]
    pub downlink_only: Option<Second>,
}

#[allow(non_camel_case_types)]
pub type Policy_Timeout = PolicyTimeout;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyStats {
    #[serde(default)]
    pub user_uplink: bool,
    #[serde(default)]
    pub user_downlink: bool,
    #[serde(default)]
    pub user_online: bool,
}

#[allow(non_camel_case_types)]
pub type Policy_Stats = PolicyStats;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyBuffer {
    /// Buffer size per connection, in bytes. -1 for unlimited buffer.
    #[serde(default)]
    pub connection: i32,
}

#[allow(non_camel_case_types)]
pub type Policy_Buffer = PolicyBuffer;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    #[serde(default)]
    pub timeout: Option<PolicyTimeout>,
    #[serde(default)]
    pub stats: Option<PolicyStats>,
    #[serde(default)]
    pub buffer: Option<PolicyBuffer>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemPolicyStats {
    #[serde(default)]
    pub inbound_uplink: bool,
    #[serde(default)]
    pub inbound_downlink: bool,
    #[serde(default)]
    pub outbound_uplink: bool,
    #[serde(default)]
    pub outbound_downlink: bool,
}

#[allow(non_camel_case_types)]
pub type SystemPolicy_Stats = SystemPolicyStats;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemPolicy {
    #[serde(default)]
    pub stats: Option<SystemPolicyStats>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub level: HashMap<u32, Policy>,
    #[serde(default)]
    pub system: Option<SystemPolicy>,
}
