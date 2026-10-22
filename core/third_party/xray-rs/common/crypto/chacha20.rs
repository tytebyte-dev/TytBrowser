// Module: common\crypto\chacha20.rs
// 1:1 Rust implementation corresponding to Go common\crypto\chacha20.go

use crate::common::errors::{Error, Result};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};

pub struct ChaCha20Cipher {
    cipher: ChaCha20Poly1305,
}

impl ChaCha20Cipher {
    pub fn new(key: &[u8; 32]) -> Self {
        let key_ref = Key::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key_ref);
        Self { cipher }
    }

    pub fn encrypt(&self, nonce: &[u8; 12], plaintext: &[u8]) -> Result<Vec<u8>> {
        let n = Nonce::from_slice(nonce);
        self.cipher
            .encrypt(n, plaintext)
            .map_err(|e| Error::Crypto(e.to_string()))
    }

    pub fn decrypt(&self, nonce: &[u8; 12], ciphertext: &[u8]) -> Result<Vec<u8>> {
        let n = Nonce::from_slice(nonce);
        self.cipher
            .decrypt(n, ciphertext)
            .map_err(|e| Error::Crypto(e.to_string()))
    }
}
