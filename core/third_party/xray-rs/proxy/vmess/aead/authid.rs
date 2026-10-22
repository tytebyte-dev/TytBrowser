// Module: proxy\vmess\aead\authid.rs
// 1:1 Rust implementation corresponding to Go proxy\vmess\aead\authid.go

use super::consts::AUTH_ID_SIZE;
use super::kdf::vmess_kdf_1;

pub struct AuthIdGenerator {
    key: [u8; 16],
}

impl AuthIdGenerator {
    pub fn new(key: &[u8]) -> Self {
        Self {
            key: vmess_kdf_1(key, b"AuthID Key"),
        }
    }

    pub fn create_auth_id(&self, timestamp: u64) -> [u8; AUTH_ID_SIZE] {
        let mut id = [0u8; AUTH_ID_SIZE];
        let ts_bytes = timestamp.to_be_bytes();
        id[..8].copy_from_slice(&ts_bytes);
        for i in 0..16 {
            id[i] ^= self.key[i];
        }
        id
    }

    pub fn matches(&self, auth_id: &[u8], current_ts: u64, window: u64) -> bool {
        if auth_id.len() != AUTH_ID_SIZE {
            return false;
        }
        let mut dec = [0u8; 16];
        for i in 0..16 {
            dec[i] = auth_id[i] ^ self.key[i];
        }
        let mut ts_buf = [0u8; 8];
        ts_buf.copy_from_slice(&dec[..8]);
        let ts = u64::from_be_bytes(ts_buf);
        ts.abs_diff(current_ts) <= window
    }
}
