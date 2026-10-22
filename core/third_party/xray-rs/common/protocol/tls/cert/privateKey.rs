// Module: common\protocol\tls\cert\privateKey.rs
// 1:1 Rust implementation corresponding to Go common\protocol\tls\cert\privateKey.go

pub struct PrivateKeyInfo {
    pub algorithm: String,
    pub raw: Vec<u8>,
}

impl PrivateKeyInfo {
    pub fn new(algorithm: impl Into<String>, raw: Vec<u8>) -> Self {
        Self {
            algorithm: algorithm.into(),
            raw,
        }
    }
}
