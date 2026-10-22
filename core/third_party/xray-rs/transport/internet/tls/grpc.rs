// Module: transport\internet\tls\grpc.rs
// 1:1 Rust implementation corresponding to Go transport\internet\tls\grpc.go

use super::config::TlsConfig;

pub const GRPC_ALPN: &[&str] = &["h2"];

#[derive(Debug, Clone)]
pub struct GrpcTlsAuthInfo {
    pub cipher_suite: u16,
    pub server_name: String,
    pub peer_certificates: Vec<Vec<u8>>,
}

impl GrpcTlsAuthInfo {
    pub fn auth_type(&self) -> &'static str {
        "tls"
    }
}

#[derive(Debug, Clone)]
pub struct GrpcTlsCredentials {
    pub config: TlsConfig,
}

impl GrpcTlsCredentials {
    pub fn new(config: TlsConfig) -> Self {
        let mut cfg = config;
        if cfg.next_protocol.is_empty() {
            cfg.next_protocol = vec!["h2".to_string()];
        }
        Self { config: cfg }
    }

    pub fn server_name(&self) -> &str {
        &self.config.server_name
    }

    pub fn override_server_name(&mut self, server_name: impl Into<String>) {
        self.config.server_name = server_name.into();
    }
}

pub fn is_grpc_alpn(alpns: &[Vec<u8>]) -> bool {
    alpns.iter().any(|a| a == b"h2")
}
