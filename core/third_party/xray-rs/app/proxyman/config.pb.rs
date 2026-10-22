// Module: app\proxyman\config.pb.rs
// 1:1 Rust protobuf message definitions corresponding to Go app\proxyman\config.pb.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InboundConfig {}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SniffingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub destination_override: Vec<String>,
    #[serde(default)]
    pub domains_excluded: Vec<String>,
    #[serde(default)]
    pub metadata_only: bool,
    #[serde(default)]
    pub route_only: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiverConfig {
    #[serde(default)]
    pub port_list: Option<Vec<u32>>,
    #[serde(default)]
    pub listen: Option<String>,
    #[serde(default)]
    pub stream_settings: Option<Vec<u8>>,
    #[serde(default)]
    pub receive_original_destination: bool,
    #[serde(default)]
    pub sniffing_settings: Option<SniffingConfig>,
    // Legacy fields for backward compatibility
    #[serde(default)]
    pub port_range: Option<String>,
    #[serde(default)]
    pub allocation_strategy: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InboundHandlerConfig {
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub receiver_settings: Option<Vec<u8>>,
    #[serde(default)]
    pub proxy_settings: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutboundConfig {}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiplexingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub concurrency: i32,
    #[serde(default)]
    pub xudp_concurrency: i32,
    #[serde(default)]
    pub xudp_proxy_udp443: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SenderConfig {
    #[serde(default)]
    pub via: Option<String>,
    #[serde(default)]
    pub stream_settings: Option<Vec<u8>>,
    #[serde(default)]
    pub proxy_settings: Option<Vec<u8>>,
    #[serde(default)]
    pub multiplex_settings: Option<MultiplexingConfig>,
    #[serde(default)]
    pub via_cidr: String,
    #[serde(default)]
    pub target_strategy: i32,
    // Legacy field for backward compatibility
    #[serde(default)]
    pub multipath_tcp: bool,
}
