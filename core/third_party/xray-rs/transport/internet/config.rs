// Module: transport\internet\config.rs
// 1:1 Rust implementation corresponding to Go transport\internet\config.go

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

pub use super::config_pb::*;
use crate::common::errors::{Error, Result};

pub type ConfigCreator = fn() -> serde_json::Value;

static CONFIG_CREATOR_REGISTRY: OnceLock<RwLock<HashMap<String, ConfigCreator>>> = OnceLock::new();

fn get_config_creator_registry() -> &'static RwLock<HashMap<String, ConfigCreator>> {
    CONFIG_CREATOR_REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn register_protocol_config_creator(
    name: impl Into<String>,
    creator: ConfigCreator,
) -> Result<()> {
    let registry = get_config_creator_registry();
    let mut map = registry
        .write()
        .map_err(|_| Error::Other("lock error".into()))?;
    let key = name.into();
    if map.contains_key(&key) {
        return Err(Error::Config(format!(
            "protocol {} already registered",
            key
        )));
    }
    map.insert(key, creator);
    Ok(())
}

pub fn create_transport_config(name: &str) -> Result<serde_json::Value> {
    let registry = get_config_creator_registry();
    let map = registry
        .read()
        .map_err(|_| Error::Other("lock error".into()))?;
    match map.get(name) {
        Some(creator) => Ok(creator()),
        None => Err(Error::NotFound(format!(
            "unknown transport protocol: {}",
            name
        ))),
    }
}

// 1:1 Go strategy table
// {strategy, prefer, fallback}
const STRATEGY_TABLE: &[[u8; 3]] = &[
    [0, 0, 0], // AsIs
    [1, 0, 0], // UseIP
    [1, 4, 0], // UseIPv4
    [1, 6, 0], // UseIPv6
    [1, 4, 6], // UseIPv4v6
    [1, 6, 4], // UseIPv6v4
    [2, 0, 0], // ForceIP
    [2, 4, 0], // ForceIPv4
    [2, 6, 0], // ForceIPv6
    [2, 4, 6], // ForceIPv4v6
    [2, 6, 4], // ForceIPv6v4
];

impl DomainStrategy {
    fn idx(self) -> usize {
        self as usize
    }

    pub fn has_strategy(self) -> bool {
        let i = self.idx();
        if i < STRATEGY_TABLE.len() {
            STRATEGY_TABLE[i][0] != 0
        } else {
            false
        }
    }

    pub fn force_ip(self) -> bool {
        let i = self.idx();
        if i < STRATEGY_TABLE.len() {
            STRATEGY_TABLE[i][0] == 2
        } else {
            false
        }
    }

    pub fn prefer_ip4(self) -> bool {
        let i = self.idx();
        if i < STRATEGY_TABLE.len() {
            STRATEGY_TABLE[i][1] == 4 || STRATEGY_TABLE[i][1] == 0
        } else {
            true
        }
    }

    pub fn prefer_ip6(self) -> bool {
        let i = self.idx();
        if i < STRATEGY_TABLE.len() {
            STRATEGY_TABLE[i][1] == 6 || STRATEGY_TABLE[i][1] == 0
        } else {
            true
        }
    }

    pub fn has_fallback(self) -> bool {
        let i = self.idx();
        if i < STRATEGY_TABLE.len() {
            STRATEGY_TABLE[i][2] != 0
        } else {
            false
        }
    }

    pub fn fallback_ip4(self) -> bool {
        let i = self.idx();
        if i < STRATEGY_TABLE.len() {
            STRATEGY_TABLE[i][2] == 4
        } else {
            false
        }
    }

    pub fn fallback_ip6(self) -> bool {
        let i = self.idx();
        if i < STRATEGY_TABLE.len() {
            STRATEGY_TABLE[i][2] == 6
        } else {
            false
        }
    }

    pub fn get_dynamic_strategy(self, is_ipv4: bool, is_ipv6: bool) -> DomainStrategy {
        match self {
            DomainStrategy::UseIp => {
                if is_ipv4 {
                    DomainStrategy::UseIp46
                } else if is_ipv6 {
                    DomainStrategy::UseIp64
                } else {
                    self
                }
            }
            DomainStrategy::ForceIp => {
                if is_ipv4 {
                    DomainStrategy::ForceIp46
                } else if is_ipv6 {
                    DomainStrategy::ForceIp64
                } else {
                    self
                }
            }
            _ => self,
        }
    }
}

impl StreamConfig {
    pub fn get_effective_protocol(&self) -> &str {
        if self.protocol_name.is_empty() {
            "tcp"
        } else {
            &self.protocol_name
        }
    }

    pub fn get_effective_transport_settings(&self) -> Option<&TransportConfig> {
        let proto = self.get_effective_protocol();
        self.transport_settings
            .iter()
            .find(|s| s.protocol_name == proto)
    }

    pub fn has_security_settings(&self) -> bool {
        !self.security_settings.is_empty()
    }
}

impl ProxyConfig {
    pub fn has_tag(&self) -> bool {
        !self.tag.is_empty()
    }
}
