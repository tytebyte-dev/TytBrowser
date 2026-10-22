// Module: transport\\internet\\grpc\\grpc.rs
// 1:1 Rust implementation corresponding to Go transport\\internet\\grpc\\grpc.go

pub use super::config::GrpcConfig;

pub const DEFAULT_SERVICE_NAME: &str = "GunService";
pub const PROTOCOL_NAME: &str = "grpc";

pub fn grpc_protocol_name() -> &'static str {
    PROTOCOL_NAME
}

pub fn default_service_name() -> &'static str {
    DEFAULT_SERVICE_NAME
}

pub struct GrpcClient {
    pub service_name: String,
}

impl GrpcClient {
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
        }
    }

    pub fn service(&self) -> &str {
        &self.service_name
    }
}
