// Module: transport\internet\finalmask\header\custom\config.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\header\custom\config.proto & config.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomHeaderConfig {
    #[serde(default)]
    pub header: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TCPItem {
    #[serde(default)]
    pub delay_min: i64,
    #[serde(default)]
    pub delay_max: i64,
    #[serde(default)]
    pub rand: i32,
    #[serde(default)]
    pub rand_min: i32,
    #[serde(default)]
    pub rand_max: i32,
    #[serde(default)]
    pub packet: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TCPSequence {
    #[serde(default)]
    pub sequence: Vec<TCPItem>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TCPConfig {
    #[serde(default)]
    pub clients: Vec<TCPSequence>,
    #[serde(default)]
    pub servers: Vec<TCPSequence>,
    #[serde(default)]
    pub errors: Vec<TCPSequence>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UDPItem {
    #[serde(default)]
    pub rand: i32,
    #[serde(default)]
    pub rand_min: i32,
    #[serde(default)]
    pub rand_max: i32,
    #[serde(default)]
    pub packet: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UDPConfig {
    #[serde(default)]
    pub client: Vec<UDPItem>,
    #[serde(default)]
    pub server: Vec<UDPItem>,
}
