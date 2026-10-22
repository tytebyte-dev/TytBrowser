// Module: transport\internet\tls\config.rs
// 1:1 Rust implementation corresponding to Go transport\internet\tls\config.go

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
#[derive(Default)]
pub enum CertificateUsage {
    #[default]
    Encipherment = 0,
    AuthorityVerify = 1,
    AuthorityIssue = 2,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Certificate {
    #[serde(default)]
    pub certificate: Vec<u8>,
    #[serde(default)]
    pub key: Vec<u8>,
    #[serde(default)]
    pub usage: CertificateUsage,
}

impl Certificate {
    pub fn new(cert: Vec<u8>, key: Vec<u8>, usage: CertificateUsage) -> Self {
        Self {
            certificate: cert,
            key,
            usage,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TlsConfig {
    #[serde(default)]
    pub allow_insecure: bool,
    #[serde(default)]
    pub certificate: Vec<Certificate>,
    #[serde(default)]
    pub server_name: String,
    #[serde(default)]
    pub next_protocol: Vec<String>,
    #[serde(default)]
    pub enable_session_resumption: bool,
    #[serde(default)]
    pub disable_system_root: bool,
    #[serde(default)]
    pub min_version: String,
    #[serde(default)]
    pub max_version: String,
    #[serde(default)]
    pub cipher_suites: String,
    #[serde(default)]
    pub fingerprint: String,
    #[serde(default)]
    pub reject_unknown_sni: bool,
    #[serde(default)]
    pub pinned_peer_certificate_chain_sha256: Vec<Vec<u8>>,
    #[serde(default)]
    pub pinned_peer_certificate_public_key_sha256: Vec<Vec<u8>>,
    #[serde(default)]
    pub master_key_log: String,
    #[serde(default)]
    pub ech_config_list: String,
    #[serde(default)]
    pub ech_server_keys: Vec<u8>,
    #[serde(default)]
    pub ech_force_query: String,

    #[serde(skip)]
    pub cert_cache: Option<Arc<Mutex<HashMap<String, Certificate>>>>,
}

impl PartialEq for TlsConfig {
    fn eq(&self, other: &Self) -> bool {
        self.allow_insecure == other.allow_insecure
            && self.certificate == other.certificate
            && self.server_name == other.server_name
            && self.next_protocol == other.next_protocol
            && self.enable_session_resumption == other.enable_session_resumption
            && self.disable_system_root == other.disable_system_root
            && self.min_version == other.min_version
            && self.max_version == other.max_version
            && self.cipher_suites == other.cipher_suites
            && self.fingerprint == other.fingerprint
            && self.reject_unknown_sni == other.reject_unknown_sni
            && self.pinned_peer_certificate_chain_sha256
                == other.pinned_peer_certificate_chain_sha256
            && self.pinned_peer_certificate_public_key_sha256
                == other.pinned_peer_certificate_public_key_sha256
            && self.master_key_log == other.master_key_log
            && self.ech_config_list == other.ech_config_list
            && self.ech_server_keys == other.ech_server_keys
            && self.ech_force_query == other.ech_force_query
    }
}

impl Eq for TlsConfig {}

impl TlsConfig {
    pub fn new(server_name: impl Into<String>) -> Self {
        Self {
            server_name: server_name.into(),
            cert_cache: Some(Arc::new(Mutex::new(HashMap::new()))),
            ..Default::default()
        }
    }

    pub fn get_or_init_cache(&mut self) -> Arc<Mutex<HashMap<String, Certificate>>> {
        if let Some(ref cache) = self.cert_cache {
            cache.clone()
        } else {
            let cache = Arc::new(Mutex::new(HashMap::new()));
            self.cert_cache = Some(cache.clone());
            cache
        }
    }

    /// Build all standard server certificates (usage == Encipherment).
    pub fn build_certificates(&self) -> Vec<Certificate> {
        self.certificate
            .iter()
            .filter(|c| c.usage == CertificateUsage::Encipherment)
            .cloned()
            .collect()
    }

    /// Get client verification root certs (usage == AuthorityVerify).
    pub fn get_client_verify_certs(&self) -> Vec<Certificate> {
        self.certificate
            .iter()
            .filter(|c| c.usage == CertificateUsage::AuthorityVerify)
            .cloned()
            .collect()
    }

    /// Issue a dynamic self-signed certificate for the given SNI if an authority issuer is configured.
    pub fn get_certificate_for_sni(&mut self, sni: &str) -> Option<Certificate> {
        let cache = self.get_or_init_cache();
        {
            let map = cache.lock().unwrap();
            if let Some(cert) = map.get(sni) {
                return Some(cert.clone());
            }
        }

        // Check if we have an AUTHORITY_ISSUE certificate
        let has_issuer = self
            .certificate
            .iter()
            .any(|c| c.usage == CertificateUsage::AuthorityIssue);
        if has_issuer
            && let Ok(generated) = rcgen::generate_simple_self_signed(vec![sni.to_string()])
        {
            let cert_pem = generated.cert.pem().into_bytes();
            let key_pem = generated.key_pair.serialize_pem().into_bytes();
            let cert = Certificate::new(cert_pem, key_pem, CertificateUsage::Encipherment);
            let mut map = cache.lock().unwrap();
            map.insert(sni.to_string(), cert.clone());
            return Some(cert);
        }

        // Fallback to static encipherment certificates
        self.build_certificates().into_iter().next()
    }
}
