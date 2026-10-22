// Module: proxy\dokodemo\config.rs
// 1:1 Rust implementation corresponding to Go proxy\dokodemo\config.go

use crate::common::net::Address;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DokodemoConfig {
    pub address: Address,
    pub port: u16,
    pub network_list: Vec<String>,
    pub timeout: u32,
    pub follow_redirect: bool,
}

impl Default for DokodemoConfig {
    fn default() -> Self {
        Self {
            address: Address::ip("127.0.0.1".parse().unwrap()),
            port: 0,
            network_list: vec!["tcp".into(), "udp".into()],
            timeout: 300,
            follow_redirect: false,
        }
    }
}
