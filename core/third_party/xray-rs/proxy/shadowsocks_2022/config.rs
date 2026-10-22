// Module: proxy\shadowsocks_2022\config.rs
// 1:1 Rust implementation corresponding to Go proxy\shadowsocks_2022\config.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shadowsocks2022Config {
    pub method: String,
    pub key: String,
    #[serde(default)]
    pub network: Option<String>,
}

pub const CIPHER_2022_BLAKE3_AES_128_GCM: &str = "2022-blake3-aes-128-gcm";
pub const CIPHER_2022_BLAKE3_AES_256_GCM: &str = "2022-blake3-aes-256-gcm";
pub const CIPHER_2022_BLAKE3_CHACHA20_POLY1305: &str = "2022-blake3-chacha20-poly1305";
