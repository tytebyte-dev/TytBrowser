// Module: common\net\network.rs
// 1:1 Rust implementation corresponding to Go common\net\network.go

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::common::errors::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Network {
    #[serde(rename = "tcp", alias = "TCP")]
    #[default]
    Tcp,
    #[serde(rename = "udp", alias = "UDP")]
    Udp,
}

impl Network {
    pub fn is_tcp(&self) -> bool {
        matches!(self, Network::Tcp)
    }

    pub fn is_udp(&self) -> bool {
        matches!(self, Network::Udp)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Network::Tcp => "tcp",
            Network::Udp => "udp",
        }
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Network {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_lowercase().as_str() {
            "tcp" => Ok(Network::Tcp),
            "udp" => Ok(Network::Udp),
            other => Err(Error::Protocol(format!("unknown network type: {}", other))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NetworkList {
    pub networks: Vec<Network>,
}

impl NetworkList {
    pub fn new() -> Self {
        Self {
            networks: Vec::new(),
        }
    }

    pub fn with_networks(networks: Vec<Network>) -> Self {
        Self { networks }
    }

    pub fn has_network(&self, network: Network) -> bool {
        self.networks.contains(&network)
    }
}
