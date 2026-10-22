// Module: proxy\shadowsocks_2022\inbound_relay.rs
// 1:1 Rust implementation corresponding to Go proxy\shadowsocks_2022\inbound_relay.go

use crate::common::errors::{Error, Result};
use crate::common::net::Destination;
use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct RelayDestination {
    pub email: String,
    pub key: String,
    pub destination: Destination,
    pub level: u32,
}

#[derive(Debug, Default)]
pub struct RelayInbound {
    destinations: RwLock<Vec<RelayDestination>>,
    method: String,
    key: String,
}

impl RelayInbound {
    pub fn new(method: impl Into<String>, key: impl Into<String>) -> Result<Self> {
        let method = method.into();
        if !method.contains("aes") {
            return Err(Error::Unsupported(format!(
                "unsupported relay method: {}",
                method
            )));
        }
        Ok(Self {
            destinations: RwLock::new(Vec::new()),
            method,
            key: key.into(),
        })
    }

    pub fn add_destination(&self, dest: RelayDestination) {
        let mut destinations = self.destinations.write().unwrap();
        destinations.push(dest);
    }

    pub fn destinations_count(&self) -> usize {
        self.destinations.read().unwrap().len()
    }

    pub fn get_destination(&self, email: &str) -> Option<RelayDestination> {
        let destinations = self.destinations.read().unwrap();
        destinations
            .iter()
            .find(|d| d.email.eq_ignore_ascii_case(email))
            .cloned()
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn key(&self) -> &str {
        &self.key
    }
}

pub use RelayInbound as Shadowsocks2022RelayInbound;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ss2022_relay_inbound() {
        let inbound = RelayInbound::new("2022-blake3-aes-128-gcm", "password123").unwrap();
        assert_eq!(inbound.destinations_count(), 0);

        inbound.add_destination(RelayDestination {
            email: "relay1".into(),
            key: "key1".into(),
            destination: Destination::default(),
            level: 0,
        });

        assert_eq!(inbound.destinations_count(), 1);
        let found = inbound.get_destination("relay1").unwrap();
        assert_eq!(found.key, "key1");

        let err_inbound = RelayInbound::new("chacha20-ietf-poly1305", "password");
        assert!(err_inbound.is_err());
    }
}
