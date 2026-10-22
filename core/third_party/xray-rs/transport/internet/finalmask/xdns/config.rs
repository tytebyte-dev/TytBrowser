// Module: transport\internet\finalmask\xdns\config.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\xdns\config.go

use super::client::XDnsClient;
use super::server::XDnsServer;
use crate::common::errors::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct XDnsConfig {
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub fake_ip: String,
}

impl XDnsConfig {
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            fake_ip: String::new(),
        }
    }

    pub fn create_client(&self) -> Result<XDnsClient> {
        XDnsClient::from_config(self)
    }

    pub fn create_server(&self) -> Result<XDnsServer> {
        XDnsServer::from_config(self)
    }
}
