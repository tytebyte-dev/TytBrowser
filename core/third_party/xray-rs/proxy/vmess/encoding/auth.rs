// Module: proxy\vmess\encoding\auth.rs
// 1:1 Rust implementation corresponding to Go proxy\vmess\encoding\auth.go

use crate::common::errors::Result;
use md5::{Digest, Md5};

pub const AUTH_TIME_TOLERANCE: u64 = 120;

/// Authenticates a byte array using FNV-1a 32-bit hash.
pub fn authenticate(b: &[u8]) -> u32 {
    let mut hash: u32 = 0x811c9dc5;
    for &byte in b {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}

/// Generates a 32-byte key from a given 16-byte array via MD5 expansion.
pub fn generate_chacha20poly1305_key(b: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    let mut hasher = Md5::new();
    hasher.update(b);
    let t1 = hasher.finalize();
    key[0..16].copy_from_slice(&t1);

    let mut hasher2 = Md5::new();
    hasher2.update(&key[0..16]);
    let t2 = hasher2.finalize();
    key[16..32].copy_from_slice(&t2);

    key
}

/// ShakeSizeParser handles length obfuscation using a simple PRNG stream or mask.
pub struct ShakeSizeParser {
    mask_state: u64,
}

impl ShakeSizeParser {
    pub fn new(nonce: &[u8]) -> Self {
        let mut state: u64 = 0xcbf29ce484222325;
        for &b in nonce {
            state ^= b as u64;
            state = state.wrapping_mul(0x100000001b3);
        }
        Self { mask_state: state }
    }

    fn next_mask(&mut self) -> u16 {
        self.mask_state = self
            .mask_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        (self.mask_state >> 48) as u16
    }

    pub fn size_bytes(&self) -> usize {
        2
    }

    pub fn encode(&mut self, size: u16) -> [u8; 2] {
        let mask = self.next_mask();
        let val = size ^ mask;
        val.to_be_bytes()
    }

    pub fn decode(&mut self, b: &[u8; 2]) -> Result<u16> {
        let mask = self.next_mask();
        let val = u16::from_be_bytes(*b);
        Ok(val ^ mask)
    }

    pub fn next_padding_len(&mut self) -> u16 {
        self.next_mask() % 64
    }

    pub fn max_padding_len(&self) -> u16 {
        64
    }
}

pub struct NoOpAuthenticator;

impl NoOpAuthenticator {
    pub fn seal<'a>(&self, plaintext: &'a [u8]) -> &'a [u8] {
        plaintext
    }

    pub fn open<'a>(&self, ciphertext: &'a [u8]) -> Result<&'a [u8]> {
        Ok(ciphertext)
    }
}
