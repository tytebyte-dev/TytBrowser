// Module: transport\internet\finalmask\mkcp\aes128gcm\config.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\mkcp\aes128gcm\config.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aes128GcmConfig {
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub key: Vec<u8>,
}

impl Aes128GcmConfig {
    pub fn new(password: impl Into<String>) -> Self {
        Self {
            password: password.into(),
            key: Vec::new(),
        }
    }
}
