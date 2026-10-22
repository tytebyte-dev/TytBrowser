// Module: proxy\shadowsocks\validator.rs
// 1:1 Rust implementation corresponding to Go proxy\shadowsocks\validator.go

use super::protocol::CipherType;

pub fn is_valid_password(cipher: CipherType, password: &str) -> bool {
    if password.is_empty() {
        return false;
    }
    match cipher {
        CipherType::None => true,
        _ => password.len() >= 4,
    }
}
