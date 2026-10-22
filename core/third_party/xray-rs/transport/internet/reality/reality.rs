// Module: transport\internet\reality\reality.rs
// 1:1 Rust implementation corresponding to Go transport\internet\reality\reality.go

use super::config::RealityConfig;
use crate::common::errors::{Error, Result};
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use hkdf::Hkdf;
use sha2::Sha256;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealityAuthSession {
    pub version: (u8, u8, u8),
    pub timestamp: u32,
    pub short_id: Vec<u8>,
}

/// Derives the 32-byte REALITY authentication key using HKDF-SHA256 with info "REALITY".
pub fn derive_auth_key(shared_secret: &[u8; 32], random_salt_20: &[u8]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(Some(random_salt_20), shared_secret);
    let mut auth_key = [0u8; 32];
    hk.expand(b"REALITY", &mut auth_key)
        .expect("32 bytes is valid length for HKDF-SHA256");
    auth_key
}

/// Seals a 16-byte session descriptor into a 32-byte REALITY TLS Session ID.
pub fn seal_session_id(
    auth_key: &[u8; 32],
    nonce_12: &[u8],
    version: (u8, u8, u8),
    timestamp: u32,
    short_id: &[u8],
    client_hello_raw: &[u8],
) -> Result<[u8; 32]> {
    let mut pt = [0u8; 16];
    pt[0] = version.0;
    pt[1] = version.1;
    pt[2] = version.2;
    pt[3] = 0; // reserved
    pt[4..8].copy_from_slice(&timestamp.to_be_bytes());

    let sid_len = std::cmp::min(8, short_id.len());
    pt[8..8 + sid_len].copy_from_slice(&short_id[..sid_len]);

    let cipher = Aes256Gcm::new_from_slice(auth_key)
        .map_err(|e| Error::Protocol(format!("Invalid AES-GCM key: {:?}", e)))?;
    let nonce = Nonce::from_slice(nonce_12);

    let ciphertext = cipher
        .encrypt(
            nonce,
            Payload {
                msg: &pt,
                aad: client_hello_raw,
            },
        )
        .map_err(|e| Error::Protocol(format!("REALITY SessionId seal error: {:?}", e)))?;

    if ciphertext.len() != 32 {
        return Err(Error::Protocol("Sealed session ID must be 32 bytes".into()));
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypts and authenticates a 32-byte REALITY TLS Session ID.
pub fn open_session_id(
    auth_key: &[u8; 32],
    nonce_12: &[u8],
    sealed_session_id: &[u8; 32],
    client_hello_raw: &[u8],
) -> Result<RealityAuthSession> {
    let cipher = Aes256Gcm::new_from_slice(auth_key)
        .map_err(|e| Error::Protocol(format!("Invalid AES-GCM key: {:?}", e)))?;
    let nonce = Nonce::from_slice(nonce_12);

    let pt = cipher
        .decrypt(
            nonce,
            Payload {
                msg: sealed_session_id,
                aad: client_hello_raw,
            },
        )
        .map_err(|e| Error::Protocol(format!("REALITY SessionId open auth error: {:?}", e)))?;

    if pt.len() != 16 {
        return Err(Error::Protocol(
            "Decrypted session plaintext must be 16 bytes".into(),
        ));
    }

    let version = (pt[0], pt[1], pt[2]);
    let timestamp = u32::from_be_bytes([pt[4], pt[5], pt[6], pt[7]]);
    let short_id = pt[8..16].to_vec();

    Ok(RealityAuthSession {
        version,
        timestamp,
        short_id,
    })
}

pub struct RealityServer {
    pub server_names: HashSet<String>,
    pub short_ids: HashSet<Vec<u8>>,
    pub private_key: Option<StaticSecret>,
}

impl RealityServer {
    pub fn new(config: &RealityConfig) -> Result<Self> {
        let server_names = config.server_names.iter().cloned().collect();
        let mut short_ids = HashSet::new();

        for sid in &config.short_ids {
            if let Ok(bytes) = hex::decode(sid) {
                short_ids.insert(bytes);
            }
        }

        let private_key = if let Some(pk_str) = &config.private_key {
            if let Ok(bytes) = hex::decode(pk_str) {
                if bytes.len() == 32 {
                    let mut arr = [0u8; 32];
                    arr.copy_from_slice(&bytes);
                    Some(StaticSecret::from(arr))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            server_names,
            short_ids,
            private_key,
        })
    }

    pub fn validate_short_id(&self, short_id: &[u8]) -> bool {
        if self.short_ids.is_empty() {
            return true;
        }
        self.short_ids.contains(short_id)
    }

    pub fn validate_server_name(&self, sni: &str) -> bool {
        if self.server_names.is_empty() {
            return true;
        }
        self.server_names.contains(sni)
    }

    /// Authenticates a ClientHello received on the server side.
    pub fn verify_client_hello(
        &self,
        client_pub: &[u8; 32],
        client_random: &[u8; 32],
        session_id: &[u8; 32],
        client_hello_raw: &[u8],
        max_time_diff_sec: u32,
    ) -> Result<RealityAuthSession> {
        let server_secret = self.private_key.as_ref().ok_or_else(|| {
            Error::Protocol("REALITY server has no private key configured".into())
        })?;

        let peer_public = PublicKey::from(*client_pub);
        let shared_secret = server_secret.diffie_hellman(&peer_public);

        let auth_key = derive_auth_key(shared_secret.as_bytes(), &client_random[..20]);
        let session = open_session_id(
            &auth_key,
            &client_random[20..],
            session_id,
            client_hello_raw,
        )?;

        // Verify timestamp difference
        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        let diff = now_unix.abs_diff(session.timestamp);

        if diff > max_time_diff_sec {
            return Err(Error::Protocol(format!(
                "REALITY handshake timestamp expired: diff {}s > max {}s",
                diff, max_time_diff_sec
            )));
        }

        // Verify short ID (match against configured non-empty short IDs)
        if !self.short_ids.is_empty() {
            let matched = self.short_ids.iter().any(|sid| {
                if sid.is_empty() {
                    return true;
                }
                session.short_id.starts_with(sid)
            });
            if !matched {
                return Err(Error::Protocol("REALITY short ID not recognized".into()));
            }
        }

        Ok(session)
    }
}

pub struct RealityClient {
    pub public_key: PublicKey,
    pub short_id: Vec<u8>,
    pub server_name: String,
}

impl RealityClient {
    pub fn new(server_name: &str, public_key_hex: &str, short_id_hex: &str) -> Result<Self> {
        let pk_bytes = hex::decode(public_key_hex)
            .map_err(|e| Error::Config(format!("Invalid REALITY public key hex: {}", e)))?;
        if pk_bytes.len() != 32 {
            return Err(Error::Config("REALITY public key must be 32 bytes".into()));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&pk_bytes);
        let public_key = PublicKey::from(arr);

        let short_id = hex::decode(short_id_hex).unwrap_or_default();

        Ok(Self {
            public_key,
            short_id,
            server_name: server_name.to_string(),
        })
    }

    /// Authenticates a ClientHello on the client side, producing the sealed 32-byte Session ID.
    pub fn auth_client_hello(
        &self,
        client_secret: EphemeralSecret,
        client_random: &[u8; 32],
        client_hello_raw: &[u8],
    ) -> Result<[u8; 32]> {
        let shared_secret = client_secret.diffie_hellman(&self.public_key);
        let auth_key = derive_auth_key(shared_secret.as_bytes(), &client_random[..20]);

        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        seal_session_id(
            &auth_key,
            &client_random[20..],
            (1, 26, 3), // Core version tuple
            now_unix,
            &self.short_id,
            client_hello_raw,
        )
    }

    pub fn generate_auth(&self) -> ([u8; 32], [u8; 32]) {
        let secret = EphemeralSecret::random_from_rng(rand::thread_rng());
        let client_pub = PublicKey::from(&secret);
        let shared_secret = secret.diffie_hellman(&self.public_key);
        (*client_pub.as_bytes(), *shared_secret.as_bytes())
    }
}
