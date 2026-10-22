// Module: proxy\vmess\aead\encrypt.rs
// 1:1 Rust implementation corresponding to Go proxy\vmess\aead\encrypt.go

use crate::common::errors::{Error, Result};
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes128Gcm, Nonce};

pub struct VmessAeadEncryptor {
    cipher: Aes128Gcm,
}

impl VmessAeadEncryptor {
    pub fn new(key: &[u8; 16]) -> Self {
        let cipher = Aes128Gcm::new_from_slice(key).unwrap();
        Self { cipher }
    }

    pub fn seal(&self, nonce: &[u8; 12], plaintext: &[u8]) -> Result<Vec<u8>> {
        let n = Nonce::from_slice(nonce);
        self.cipher
            .encrypt(n, plaintext)
            .map_err(|e| Error::Crypto(e.to_string()))
    }

    pub fn open(&self, nonce: &[u8; 12], ciphertext: &[u8]) -> Result<Vec<u8>> {
        let n = Nonce::from_slice(nonce);
        self.cipher
            .decrypt(n, ciphertext)
            .map_err(|e| Error::Crypto(e.to_string()))
    }
}
