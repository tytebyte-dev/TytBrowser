// Module: common\protocol\tls\cert\cert.rs
// 1:1 Rust implementation corresponding to Go common\protocol\tls\cert\cert.go

use crate::common::errors::{Error, Result};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Certificate {
    pub certificate: Vec<u8>,
    pub private_key: Vec<u8>,
}

impl Certificate {
    pub fn new(certificate: Vec<u8>, private_key: Vec<u8>) -> Self {
        Self {
            certificate,
            private_key,
        }
    }

    pub fn sha256_fingerprint(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.certificate);
        let res = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&res);
        out
    }

    pub fn to_pem(&self) -> (String, String) {
        let cert_pem = format!(
            "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n",
            base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &self.certificate
            )
        );
        let key_pem = format!(
            "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----\n",
            base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &self.private_key
            )
        );
        (cert_pem, key_pem)
    }
}

pub fn parse_certificate(cert_pem: &str, key_pem: &str) -> Result<Certificate> {
    let extract_base64 = |pem: &str| -> Result<Vec<u8>> {
        let lines: Vec<&str> = pem
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.starts_with("-----"))
            .collect();
        let b64 = lines.join("");
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &b64)
            .map_err(|e| Error::Protocol(format!("failed to decode pem base64: {}", e)))
    };

    let cert = extract_base64(cert_pem)?;
    let key = extract_base64(key_pem)?;
    Ok(Certificate::new(cert, key))
}
