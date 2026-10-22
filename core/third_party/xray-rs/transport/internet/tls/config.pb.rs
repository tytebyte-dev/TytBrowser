// Module: transport\internet\tls\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Certificate {
    pub certificate: Vec<u8>,
    pub key: Vec<u8>,
    pub usage: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub allow_insecure: bool,
    pub certificate: Vec<Certificate>,
    pub server_name: String,
    pub next_protocol: Vec<String>,
    pub enable_session_resumption: bool,
    pub disable_system_root: bool,
    pub min_version: String,
    pub max_version: String,
    pub cipher_suites: String,
    pub fingerprint: String,
    pub reject_unknown_sni: bool,
    pub pinned_peer_certificate_chain_sha256: Vec<Vec<u8>>,
    pub pinned_peer_certificate_public_key_sha256: Vec<Vec<u8>>,
    pub master_key_log: String,
}
