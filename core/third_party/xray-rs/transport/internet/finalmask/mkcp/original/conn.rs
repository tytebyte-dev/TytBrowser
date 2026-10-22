// Module: transport\internet\finalmask\mkcp\original\conn.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\mkcp\original\conn.go

use super::config::OriginalConfig;
use super::xor::{xorbkd, xorfwd};
use crate::common::errors::{Error, Result};

/// FNV-1a 32-bit hash calculation matching Go's hash/fnv
pub fn fnv32a(data: &[u8]) -> u32 {
    const FNV_OFFSET_BASIS: u32 = 2166136261;
    const FNV_PRIME: u32 = 16777619;
    let mut hash = FNV_OFFSET_BASIS;
    for &byte in data {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Simple AEAD-like authenticator and obfuscator for mKCP original mode.
#[derive(Debug, Clone, Default)]
pub struct SimpleAead;

impl SimpleAead {
    pub fn new() -> Self {
        Self
    }

    pub fn nonce_size(&self) -> usize {
        0
    }

    pub fn overhead(&self) -> usize {
        6
    }

    /// Seals plaintext into a self-authenticating obfuscated datagram.
    pub fn seal(&self, plain: &[u8]) -> Vec<u8> {
        let mut dst = Vec::with_capacity(6 + plain.len() + 4);
        dst.extend_from_slice(&[0, 0, 0, 0]); // FNV-1a placeholder
        dst.extend_from_slice(&(plain.len() as u16).to_be_bytes());
        dst.extend_from_slice(plain);

        // FNV-1a over dst[4..] (length + plain)
        let hash = fnv32a(&dst[4..]);
        dst[0..4].copy_from_slice(&hash.to_be_bytes());

        let dst_len = dst.len();
        let xtra = 4 - (dst_len % 4);
        if xtra != 4 {
            dst.resize(dst_len + xtra, 0);
        }

        xorfwd(&mut dst);

        if xtra != 4 {
            dst.truncate(dst_len);
        }

        dst
    }

    /// Opens an obfuscated datagram and verifies FNV-1a checksum and length.
    pub fn open(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() < 6 {
            return Err(Error::Protocol("ciphertext too short (< 6 bytes)".into()));
        }

        let mut dst = ciphertext.to_vec();
        let dst_len = dst.len();
        let xtra = 4 - (dst_len % 4);
        if xtra != 4 {
            dst.resize(dst_len + xtra, 0);
        }

        xorbkd(&mut dst);

        if xtra != 4 {
            dst.truncate(dst_len);
        }

        let expected_hash = fnv32a(&dst[4..]);
        let actual_hash = u32::from_be_bytes([dst[0], dst[1], dst[2], dst[3]]);
        if actual_hash != expected_hash {
            return Err(Error::Protocol("invalid auth".into()));
        }

        let length = u16::from_be_bytes([dst[4], dst[5]]) as usize;
        if dst.len() - 6 != length {
            return Err(Error::Protocol("invalid auth".into()));
        }

        Ok(dst[6..].to_vec())
    }
}

/// Packet-level connection wrapper for original mode.
pub struct OriginalPacketConn {
    aead: SimpleAead,
}

impl OriginalPacketConn {
    pub fn new(_config: &OriginalConfig) -> Self {
        Self {
            aead: SimpleAead::new(),
        }
    }

    pub fn size(&self) -> usize {
        self.aead.overhead()
    }

    pub fn wrap(&self, payload: &[u8]) -> Vec<u8> {
        self.aead.seal(payload)
    }

    pub fn unwrap(&self, packet: &[u8]) -> Result<Vec<u8>> {
        self.aead.open(packet)
    }
}

impl Default for OriginalPacketConn {
    fn default() -> Self {
        Self::new(&OriginalConfig::default())
    }
}
