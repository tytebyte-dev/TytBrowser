// Module: transport\internet\tls\ech.rs
// 1:1 Rust implementation corresponding to Go transport\internet\tls\ech.go

use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const DEFAULT_ECH_TTL: Duration = Duration::from_secs(3600);
pub const SVCB_PARAM_ECH: u16 = 5;
pub const DUMMY_FALLBACK_ECH_CONFIG: &[u8] = &[1, 1, 4, 5, 1, 4];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EchServerKey {
    pub private_key: Vec<u8>,
    pub config: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EchConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub config_list: String,
    #[serde(default)]
    pub force_query: String, // "none", "half", "full" (default)
    #[serde(default)]
    pub server_keys: Vec<u8>,
    #[serde(default)]
    pub outer_sni: Option<String>,
}

impl EchConfig {
    pub fn new(enabled: bool, outer_sni: Option<String>) -> Self {
        Self {
            enabled,
            config_list: String::new(),
            force_query: "full".to_string(),
            server_keys: Vec::new(),
            outer_sni,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EchConfigRecord {
    pub config: Vec<u8>,
    pub expire: Instant,
    pub err: Option<String>,
}

pub struct EchConfigCache {
    records: Mutex<HashMap<String, EchConfigRecord>>,
}

impl Default for EchConfigCache {
    fn default() -> Self {
        Self::new()
    }
}

impl EchConfigCache {
    pub fn new() -> Self {
        Self {
            records: Mutex::new(HashMap::new()),
        }
    }

    pub fn make_key(server: &str, domain: &str) -> String {
        format!("{}|{}", server, domain)
    }

    pub fn get(&self, server: &str, domain: &str) -> Option<Vec<u8>> {
        let key = Self::make_key(server, domain);
        let mut map = self.records.lock().unwrap();
        if let Some(record) = map.get(&key) {
            if Instant::now() < record.expire && record.err.is_none() {
                return Some(record.config.clone());
            }
            if Instant::now() >= record.expire {
                map.remove(&key);
            }
        }
        None
    }

    pub fn store(&self, server: &str, domain: &str, config: Vec<u8>, ttl: Duration) {
        let key = Self::make_key(server, domain);
        let mut map = self.records.lock().unwrap();
        map.insert(
            key,
            EchConfigRecord {
                config,
                expire: Instant::now() + ttl,
                err: None,
            },
        );
    }

    pub fn store_error(&self, server: &str, domain: &str, err: String, ttl: Duration) {
        let key = Self::make_key(server, domain);
        let mut map = self.records.lock().unwrap();
        map.insert(
            key,
            EchConfigRecord {
                config: Vec::new(),
                expire: Instant::now() + ttl,
                err: Some(err),
            },
        );
    }

    pub fn clear(&self) {
        self.records.lock().unwrap().clear();
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_ECH_CACHE: Arc<EchConfigCache> = Arc::new(EchConfigCache::new());
}

/// Convert raw binary ECH keys to Go/RFC-compatible ECH private key & config pairs.
/// Wire format per entry:
/// [u16 private_key_len, private_key_bytes, u16 config_len, config_bytes]
pub fn convert_to_ech_keys(data: &[u8]) -> Result<Vec<EchServerKey>, &'static str> {
    let mut keys = Vec::new();
    let mut offset = 0;

    while offset < data.len() {
        if offset + 2 > data.len() {
            return Err("goech: invalid length");
        }
        let sk_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;

        if offset + sk_len + 2 > data.len() {
            return Err("goech: invalid length");
        }
        let sk = data[offset..offset + sk_len].to_vec();
        offset += sk_len;

        let cfg_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;

        if offset + cfg_len > data.len() {
            return Err("goech: invalid length");
        }
        let cfg = data[offset..offset + cfg_len].to_vec();
        offset += cfg_len;

        keys.push(EchServerKey {
            private_key: sk,
            config: cfg,
        });
    }

    Ok(keys)
}

/// Encode ECH server keys into wire format.
pub fn encode_ech_keys(keys: &[EchServerKey]) -> Vec<u8> {
    let mut buf = Vec::new();
    for k in keys {
        buf.extend_from_slice(&(k.private_key.len() as u16).to_be_bytes());
        buf.extend_from_slice(&k.private_key);
        buf.extend_from_slice(&(k.config.len() as u16).to_be_bytes());
        buf.extend_from_slice(&k.config);
    }
    buf
}

/// Parse RFC 9460 SVCB/HTTPS SvcParams to extract parameter key 5 (ECHConfigList).
pub fn parse_svcb_ech_param(svcb_params: &[u8]) -> Option<Vec<u8>> {
    let mut offset = 0;
    while offset + 4 <= svcb_params.len() {
        let key = u16::from_be_bytes([svcb_params[offset], svcb_params[offset + 1]]);
        let len = u16::from_be_bytes([svcb_params[offset + 2], svcb_params[offset + 3]]) as usize;
        offset += 4;
        if offset + len > svcb_params.len() {
            return None;
        }
        if key == SVCB_PARAM_ECH {
            return Some(svcb_params[offset..offset + len].to_vec());
        }
        offset += len;
    }
    None
}

/// Encode an RFC 9460 SVCB SvcParam with key 5 (ECH).
pub fn encode_svcb_ech_param(ech_config: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(4 + ech_config.len());
    buf.extend_from_slice(&SVCB_PARAM_ECH.to_be_bytes());
    buf.extend_from_slice(&(ech_config.len() as u16).to_be_bytes());
    buf.extend_from_slice(ech_config);
    buf
}

/// Resolve ECH config bytes from configuration or cache, with fallback enforcement.
pub fn resolve_ech_config(
    ech_config_list: &str,
    server_name: &str,
    force_query: &str,
) -> Result<Option<Vec<u8>>, String> {
    if ech_config_list.is_empty() {
        return Ok(None);
    }

    let mode = if force_query.is_empty() {
        "full"
    } else {
        force_query
    };

    let result = if ech_config_list.contains("://") {
        // Query format: "example.com+https://1.1.1.1/dns-query" or "udp://1.1.1.1"
        let parts: Vec<&str> = ech_config_list.split('+').collect();
        let (name_to_query, dns_server) = match parts.len() {
            2 => (parts[0], parts[1]),
            1 => (server_name, parts[0]),
            _ => {
                return Err(format!(
                    "Invalid ECH DNS server format: {}",
                    ech_config_list
                ));
            }
        };

        if name_to_query.is_empty() {
            return Err(
                "Using DNS for ECH Config needs serverName or example.com+server format".into(),
            );
        }

        // Check global cache
        GLOBAL_ECH_CACHE.get(dns_server, name_to_query)
    } else {
        // Direct Base64 encoded ECHConfigList
        match base64::engine::general_purpose::STANDARD.decode(ech_config_list.trim()) {
            Ok(bytes) => Some(bytes),
            Err(e) => return Err(format!("Failed to unmarshal ECHConfigList: {}", e)),
        }
    };

    match result {
        Some(cfg) if !cfg.is_empty() => Ok(Some(cfg)),
        _ => {
            if mode == "full" {
                // To prevent SNI leakage on failed ECH query, supply dummy config that triggers handshake failure
                Ok(Some(DUMMY_FALLBACK_ECH_CONFIG.to_vec()))
            } else {
                Ok(None)
            }
        }
    }
}
