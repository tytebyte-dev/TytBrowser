// Module: transport\internet\sockopt.rs
// 1:1 Rust implementation corresponding to Go transport\internet\sockopt.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SocketOptions {
    #[serde(default)]
    pub mark: u32,
    #[serde(default)]
    pub tfo: i32,
    #[serde(default)]
    pub tcp_keep_alive_interval: i32,
    #[serde(default)]
    pub tcp_keep_alive_idle: i32,
    #[serde(default)]
    pub tcp_congestion: Option<String>,
    #[serde(default)]
    pub interface: Option<String>,
    #[serde(default)]
    pub receive_original_dest_address: bool,
}

pub type SocketConfig = SocketOptions;

impl SocketOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_mark(mut self, mark: u32) -> Self {
        self.mark = mark;
        self
    }

    pub fn with_tfo(mut self, tfo: i32) -> Self {
        self.tfo = tfo;
        self
    }

    pub fn parse_tfo_value(&self) -> i32 {
        if self.tfo == 0 {
            -1
        } else if self.tfo < 0 {
            0
        } else {
            self.tfo
        }
    }
}

pub fn is_tcp_socket(network: &str) -> bool {
    matches!(network, "tcp" | "tcp4" | "tcp6")
}

pub fn is_udp_socket(network: &str) -> bool {
    matches!(network, "udp" | "udp4" | "udp6")
}
