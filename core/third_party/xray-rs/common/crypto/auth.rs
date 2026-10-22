// Module: common\crypto\auth.rs
// 1:1 Rust implementation corresponding to Go common\crypto\auth.go

use crate::common::errors::{Error, Result};
use hkdf::hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub struct Authentication;

impl Authentication {
    pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<[u8; 32]> {
        let mut mac = HmacSha256::new_from_slice(key).map_err(|e| Error::Crypto(e.to_string()))?;
        mac.update(data);
        let result = mac.finalize().into_bytes();
        let mut out = [0u8; 32];
        out.copy_from_slice(&result);
        Ok(out)
    }

    pub fn verify_hmac_sha256(key: &[u8], data: &[u8], expected: &[u8]) -> bool {
        if let Ok(mut mac) = HmacSha256::new_from_slice(key) {
            mac.update(data);
            mac.verify_slice(expected).is_ok()
        } else {
            false
        }
    }
}
