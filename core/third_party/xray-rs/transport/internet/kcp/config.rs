// Module: transport\internet\kcp\config.rs
// 1:1 Rust implementation corresponding to Go transport\internet\kcp\config.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KcpConfig {
    pub mtu: u32,
    pub tti: u32,
    pub uplink_capacity: u32,
    pub downlink_capacity: u32,
    pub congestion: bool,
    pub read_buffer_size: u32,
    pub write_buffer_size: u32,
    pub seed: Option<String>,
}

impl Default for KcpConfig {
    fn default() -> Self {
        Self {
            mtu: 1350,
            tti: 50,
            uplink_capacity: 5,
            downlink_capacity: 20,
            congestion: false,
            read_buffer_size: 2 * 1024 * 1024,
            write_buffer_size: 2 * 1024 * 1024,
            seed: None,
        }
    }
}

impl KcpConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sending_in_flight_size(&self) -> u32 {
        let size = self.uplink_capacity * 1024 * 1024 / self.mtu / (1000 / self.tti);
        size.max(8)
    }

    pub fn receiving_in_flight_size(&self) -> u32 {
        let size = self.downlink_capacity * 1024 * 1024 / self.mtu / (1000 / self.tti);
        size.max(8)
    }
}
