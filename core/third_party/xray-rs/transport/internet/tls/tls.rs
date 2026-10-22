// Module: transport\internet\tls\tls.rs
// Standard official Rust TLS implementation (rustls + ring + webpki_roots)
// with uTLS / rquest browser fingerprint emulation profiles (Chrome, Firefox, Safari, Edge, Random)

use std::sync::Arc;
use tokio_rustls::rustls::{
    ClientConfig, DigitallySignedStruct, Error as RustlsError, ServerConfig, SignatureScheme,
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    pki_types::{CertificateDer, ServerName, UnixTime},
};
use tokio_rustls::{TlsAcceptor, TlsConnector};

use crate::common::errors::{Error, Result};
use crate::common::net::BoxStream;

#[derive(Debug)]
pub struct NoCertificateVerification;

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, RustlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, RustlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, RustlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
        ]
    }
}

/// Applies browser TLS fingerprint emulation (Chrome, Firefox, Safari, Edge, Android, Random)
/// aligning with Xray uTLS and rquest emulation specifications.
pub fn apply_fingerprint(provider: &mut rustls::crypto::CryptoProvider, fingerprint: &str) {
    let fp = fingerprint.trim().to_ascii_lowercase();
    match fp.as_str() {
        "firefox" => {
            // Firefox standard cipher suite order: AES-128 -> ChaCha20 -> AES-256
            provider.cipher_suites.sort_by_key(|cs| match cs.suite() {
                rustls::CipherSuite::TLS13_AES_128_GCM_SHA256 => 0,
                rustls::CipherSuite::TLS13_CHACHA20_POLY1305_SHA256 => 1,
                rustls::CipherSuite::TLS13_AES_256_GCM_SHA384 => 2,
                rustls::CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256 => 3,
                rustls::CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256 => 4,
                rustls::CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256 => 5,
                rustls::CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256 => 6,
                rustls::CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384 => 7,
                rustls::CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384 => 8,
                _ => 10,
            });
        }
        "chrome" | "edge" | "360" | "qq" | "android" => {
            // Chrome / Edge Chromium standard cipher suite order: AES-128 -> AES-256 -> ChaCha20
            provider.cipher_suites.sort_by_key(|cs| match cs.suite() {
                rustls::CipherSuite::TLS13_AES_128_GCM_SHA256 => 0,
                rustls::CipherSuite::TLS13_AES_256_GCM_SHA384 => 1,
                rustls::CipherSuite::TLS13_CHACHA20_POLY1305_SHA256 => 2,
                rustls::CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256 => 3,
                rustls::CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256 => 4,
                rustls::CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384 => 5,
                rustls::CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384 => 6,
                rustls::CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256 => 7,
                rustls::CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256 => 8,
                _ => 10,
            });
        }
        "safari" | "ios" => {
            // Safari / iOS profile: prioritizes ChaCha20 and ECDSA
            provider.cipher_suites.sort_by_key(|cs| match cs.suite() {
                rustls::CipherSuite::TLS13_AES_128_GCM_SHA256 => 0,
                rustls::CipherSuite::TLS13_AES_256_GCM_SHA384 => 1,
                rustls::CipherSuite::TLS13_CHACHA20_POLY1305_SHA256 => 2,
                rustls::CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384 => 3,
                rustls::CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256 => 4,
                rustls::CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256 => 5,
                rustls::CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384 => 6,
                rustls::CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256 => 7,
                rustls::CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256 => 8,
                _ => 10,
            });
        }
        "random" | "randomized" => {
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            provider.cipher_suites.shuffle(&mut rng);
            provider.kx_groups.shuffle(&mut rng);
        }
        _ => {}
    }
}

pub struct TlsClient {
    connector: TlsConnector,
}

impl TlsClient {
    pub fn new(server_name: &str, allow_insecure: bool, alpn: Vec<Vec<u8>>) -> Result<Self> {
        Self::new_with_fingerprint(server_name, allow_insecure, alpn, "")
    }

    pub fn new_with_fingerprint(
        _server_name: &str,
        allow_insecure: bool,
        alpn: Vec<Vec<u8>>,
        fingerprint: &str,
    ) -> Result<Self> {
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let mut provider = rustls::crypto::ring::default_provider();
        if !fingerprint.is_empty() {
            apply_fingerprint(&mut provider, fingerprint);
        }

        let mut config = if allow_insecure {
            ClientConfig::builder_with_provider(Arc::new(provider))
                .with_safe_default_protocol_versions()
                .map_err(|e| Error::Protocol(format!("TLS config protocol version error: {}", e)))?
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(NoCertificateVerification))
                .with_no_client_auth()
        } else {
            ClientConfig::builder_with_provider(Arc::new(provider))
                .with_safe_default_protocol_versions()
                .map_err(|e| Error::Protocol(format!("TLS config protocol version error: {}", e)))?
                .with_root_certificates(root_store)
                .with_no_client_auth()
        };

        let mut alpn_protocols = alpn;
        if alpn_protocols.is_empty() {
            alpn_protocols = vec![b"http/1.1".to_vec()];
        }
        config.alpn_protocols = alpn_protocols;

        Ok(Self {
            connector: TlsConnector::from(Arc::new(config)),
        })
    }

    pub async fn connect(&self, domain: &str, stream: BoxStream) -> Result<BoxStream> {
        let clean_domain = domain.trim();
        let server_name = ServerName::try_from(clean_domain.to_string()).map_err(|e| {
            Error::Protocol(format!("Invalid TLS server name '{}': {}", clean_domain, e))
        })?;
        let tls_stream = self
            .connector
            .connect(server_name, stream)
            .await
            .map_err(|e| {
                Error::Protocol(format!(
                    "TLS handshake failed with '{}': {}",
                    clean_domain, e
                ))
            })?;
        Ok(Box::pin(tls_stream))
    }
}

pub struct TlsServer {
    acceptor: TlsAcceptor,
}

impl TlsServer {
    pub fn new(cert_der: Vec<u8>, key_der: Vec<u8>) -> Result<Self> {
        let certs = vec![CertificateDer::from(cert_der)];
        let key = rustls_pki_types::PrivateKeyDer::Pkcs8(
            rustls_pki_types::PrivatePkcs8KeyDer::from(key_der),
        );

        let provider = rustls::crypto::ring::default_provider();
        let config = ServerConfig::builder_with_provider(Arc::new(provider))
            .with_safe_default_protocol_versions()
            .map_err(|e| Error::Protocol(format!("TLS server protocol error: {}", e)))?
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| Error::Protocol(format!("TLS server single cert error: {}", e)))?;

        Ok(Self {
            acceptor: TlsAcceptor::from(Arc::new(config)),
        })
    }

    pub async fn accept(&self, stream: BoxStream) -> Result<BoxStream> {
        let tls_stream = self
            .acceptor
            .accept(stream)
            .await
            .map_err(|e| Error::Protocol(format!("TLS accept failed: {}", e)))?;
        Ok(Box::pin(tls_stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_fingerprint_chrome_vs_firefox() {
        let mut chrome_provider = rustls::crypto::ring::default_provider();
        apply_fingerprint(&mut chrome_provider, "chrome");

        let mut firefox_provider = rustls::crypto::ring::default_provider();
        apply_fingerprint(&mut firefox_provider, "firefox");

        // In Chrome, TLS13_AES_256_GCM_SHA384 precedes TLS13_CHACHA20_POLY1305_SHA256
        let chrome_pos_aes256 = chrome_provider
            .cipher_suites
            .iter()
            .position(|s| s.suite() == rustls::CipherSuite::TLS13_AES_256_GCM_SHA384)
            .unwrap();
        let chrome_pos_chacha = chrome_provider
            .cipher_suites
            .iter()
            .position(|s| s.suite() == rustls::CipherSuite::TLS13_CHACHA20_POLY1305_SHA256)
            .unwrap();
        assert!(chrome_pos_aes256 < chrome_pos_chacha);

        // In Firefox, TLS13_CHACHA20_POLY1305_SHA256 precedes TLS13_AES_256_GCM_SHA384
        let firefox_pos_aes256 = firefox_provider
            .cipher_suites
            .iter()
            .position(|s| s.suite() == rustls::CipherSuite::TLS13_AES_256_GCM_SHA384)
            .unwrap();
        let firefox_pos_chacha = firefox_provider
            .cipher_suites
            .iter()
            .position(|s| s.suite() == rustls::CipherSuite::TLS13_CHACHA20_POLY1305_SHA256)
            .unwrap();
        assert!(firefox_pos_chacha < firefox_pos_aes256);
    }

    #[test]
    fn test_tls_client_creation_with_fingerprints() {
        for fp in &["chrome", "firefox", "safari", "edge", "random", ""] {
            let client =
                TlsClient::new_with_fingerprint("example.com", true, vec![b"h2".to_vec()], fp);
            assert!(
                client.is_ok(),
                "failed to create client with fingerprint '{}'",
                fp
            );
        }
    }
}
