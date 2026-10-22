// Module: proxy\shadowsocks\shadowsocks.rs
// 1:1 Rust implementation corresponding to Go proxy\shadowsocks\shadowsocks.go

pub const PROTOCOL_NAME: &str = "shadowsocks";
pub const CIPHER_AES_128_GCM: &str = "aes-128-gcm";
pub const CIPHER_AES_256_GCM: &str = "aes-256-gcm";
pub const CIPHER_CHACHA20_POLY1305: &str = "chacha20-poly1305";
pub const CIPHER_2022_BLAKE3_AES_128_GCM: &str = "2022-blake3-aes-128-gcm";
pub const CIPHER_2022_BLAKE3_AES_256_GCM: &str = "2022-blake3-aes-256-gcm";
