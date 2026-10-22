// Module: transport\internet\finalmask\mkcp\aes128gcm\conn.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\mkcp\aes128gcm\conn.go

use super::config::Aes128GcmConfig;
use crate::common::errors::{Error, Result};
use aes_gcm::{
    Aes128Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;
use sha2::{Digest, Sha256};

pub const GCM_NONCE_SIZE: usize = 12;
pub const GCM_TAG_SIZE: usize = 16;

pub struct Aes128GcmPacketConn {
    cipher: Aes128Gcm,
}

impl Aes128GcmPacketConn {
    pub fn new(password: &str) -> Self {
        let hash = Sha256::digest(password.as_bytes());
        let key = &hash[..16];
        let cipher = Aes128Gcm::new_from_slice(key).expect("16-byte key is valid for AES-128");
        Self { cipher }
    }

    pub fn from_config(config: &Aes128GcmConfig) -> Self {
        if !config.key.is_empty() && config.key.len() >= 16 {
            let cipher =
                Aes128Gcm::new_from_slice(&config.key[..16]).expect("16-byte key is valid");
            Self { cipher }
        } else {
            Self::new(&config.password)
        }
    }

    pub fn size(&self) -> usize {
        GCM_NONCE_SIZE
    }

    pub fn overhead(&self) -> usize {
        GCM_TAG_SIZE
    }

    /// Wraps (seals) a plaintext packet with a fresh random 12-byte nonce.
    pub fn wrap(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; GCM_NONCE_SIZE];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| Error::Protocol(format!("aes128gcm seal error: {:?}", e)))?;

        let mut out = Vec::with_capacity(GCM_NONCE_SIZE + ciphertext.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    /// Unwraps (opens) an incoming datagram containing nonce + ciphertext + tag.
    pub fn unwrap(&self, packet: &[u8]) -> Result<Vec<u8>> {
        if packet.len() < GCM_NONCE_SIZE + GCM_TAG_SIZE {
            return Err(Error::Protocol("aead short length".into()));
        }

        let (nonce_bytes, ciphertext) = packet.split_at(GCM_NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| Error::Protocol(format!("aead open: {:?}", e)))
    }
}
