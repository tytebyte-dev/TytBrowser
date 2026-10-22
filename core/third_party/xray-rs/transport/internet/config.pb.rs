// Module: transport\internet\config.pb.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i32)]
#[derive(Default)]
pub enum DomainStrategy {
    #[default]
    AsIs = 0,
    UseIp = 1,
    UseIp4 = 2,
    UseIp6 = 3,
    UseIp46 = 4,
    UseIp64 = 5,
    ForceIp = 6,
    ForceIp4 = 7,
    ForceIp6 = 8,
    ForceIp46 = 9,
    ForceIp64 = 10,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i32)]
#[derive(Default)]
pub enum AddressPortStrategy {
    #[default]
    None = 0,
    SrvPortOnly = 1,
    SrvAddressOnly = 2,
    SrvPortAndAddress = 3,
    TxtPortOnly = 4,
    TxtAddressOnly = 5,
    TxtPortAndAddress = 6,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportConfig {
    #[serde(default)]
    pub protocol_name: String,
    #[serde(default)]
    pub settings: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UdpHop {
    #[serde(default)]
    pub ports: Vec<u32>,
    #[serde(default)]
    pub interval_min: i64,
    #[serde(default)]
    pub interval_max: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuicParams {
    #[serde(default)]
    pub congestion: String,
    #[serde(default)]
    pub brutal_up: u64,
    #[serde(default)]
    pub brutal_down: u64,
    pub udp_hop: Option<UdpHop>,
    #[serde(default)]
    pub init_stream_receive_window: u64,
    #[serde(default)]
    pub max_stream_receive_window: u64,
    #[serde(default)]
    pub init_conn_receive_window: u64,
    #[serde(default)]
    pub max_conn_receive_window: u64,
    #[serde(default)]
    pub max_idle_timeout: i64,
    #[serde(default)]
    pub keep_alive_period: i64,
    #[serde(default)]
    pub disable_path_mtu_discovery: bool,
    #[serde(default)]
    pub max_incoming_streams: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyConfig {
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub transport_layer_proxy: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomSockopt {
    pub system: String,
    pub network: String,
    pub level: String,
    pub opt: String,
    pub value: String,
    #[serde(rename = "type")]
    pub opt_type: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HappyEyeballsConfig {
    #[serde(default)]
    pub prioritize_ipv6: bool,
    #[serde(default)]
    pub interleave: u32,
    #[serde(default)]
    pub try_delay_ms: u64,
    #[serde(default)]
    pub max_concurrent_try: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamConfig {
    #[serde(default)]
    pub protocol_name: String,
    #[serde(default)]
    pub transport_settings: Vec<TransportConfig>,
    #[serde(default)]
    pub security_type: String,
    #[serde(default)]
    pub security_settings: Vec<serde_json::Value>,
    pub quic_params: Option<QuicParams>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}
