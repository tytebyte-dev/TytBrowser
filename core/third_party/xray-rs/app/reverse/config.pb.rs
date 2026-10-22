// Module: app\reverse\config.pb.rs
// 1:1 Rust protobuf message definitions corresponding to Go app\reverse\config.pb.go

use serde::{Deserialize, Serialize};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Control_State {
    #[default]
    Active = 0,
    Drain = 1,
}

pub type ControlState = Control_State;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Control {
    #[serde(default)]
    pub state: Control_State,
    #[serde(default)]
    pub random: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeConfig {
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub domain: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortalConfig {
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub domain: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub bridge_config: Vec<BridgeConfig>,
    #[serde(default)]
    pub portal_config: Vec<PortalConfig>,
}
