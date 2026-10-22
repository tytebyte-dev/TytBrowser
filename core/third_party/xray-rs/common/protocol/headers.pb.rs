// Module: common\protocol\headers.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityType {
    Unknown = 0,
    Legacy = 1,
    Auto = 2,
    Aes128Gcm = 3,
    Chacha20Poly1305 = 4,
    None = 5,
    Zero = 6,
}

impl Default for SecurityType {
    fn default() -> Self {
        Self::Auto
    }
}
