use crate::common::errors::{Error, Result};
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes128Gcm, Aes256Gcm, Nonce};

pub struct AesGcmCipher;

impl AesGcmCipher {
    pub fn encrypt_128(key: &[u8], nonce: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
        if key.len() != 16 {
            return Err(Error::Crypto("AES-128 key must be 16 bytes".into()));
        }
        if nonce.len() != 12 {
            return Err(Error::Crypto("AES-GCM nonce must be 12 bytes".into()));
        }
        let cipher = Aes128Gcm::new_from_slice(key).map_err(|e| Error::Crypto(e.to_string()))?;
        let nonce = Nonce::from_slice(nonce);
        cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| Error::Crypto(e.to_string()))
    }

    pub fn decrypt_128(key: &[u8], nonce: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
        if key.len() != 16 {
            return Err(Error::Crypto("AES-128 key must be 16 bytes".into()));
        }
        if nonce.len() != 12 {
            return Err(Error::Crypto("AES-GCM nonce must be 12 bytes".into()));
        }
        let cipher = Aes128Gcm::new_from_slice(key).map_err(|e| Error::Crypto(e.to_string()))?;
        let nonce = Nonce::from_slice(nonce);
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| Error::Crypto(e.to_string()))
    }

    pub fn encrypt_256(key: &[u8], nonce: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
        if key.len() != 32 {
            return Err(Error::Crypto("AES-256 key must be 32 bytes".into()));
        }
        if nonce.len() != 12 {
            return Err(Error::Crypto("AES-GCM nonce must be 12 bytes".into()));
        }
        let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| Error::Crypto(e.to_string()))?;
        let nonce = Nonce::from_slice(nonce);
        cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| Error::Crypto(e.to_string()))
    }

    pub fn decrypt_256(key: &[u8], nonce: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
        if key.len() != 32 {
            return Err(Error::Crypto("AES-256 key must be 32 bytes".into()));
        }
        if nonce.len() != 12 {
            return Err(Error::Crypto("AES-GCM nonce must be 12 bytes".into()));
        }
        let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| Error::Crypto(e.to_string()))?;
        let nonce = Nonce::from_slice(nonce);
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| Error::Crypto(e.to_string()))
    }
}

pub fn new_aes_gcm(key: &[u8]) -> Result<AesGcmCipher> {
    if key.len() != 16 && key.len() != 32 {
        return Err(Error::Crypto("AES key must be 16 or 32 bytes".into()));
    }
    Ok(AesGcmCipher)
}
