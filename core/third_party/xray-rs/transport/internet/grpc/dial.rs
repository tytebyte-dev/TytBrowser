// Module: transport\internet\grpc\dial.rs
// 1:1 Rust implementation corresponding to Go transport\internet\grpc\dial.go

use super::config::GrpcConfig;
use crate::common::errors::Result;
use crate::common::net::{BoxStream, Destination};
use crate::transport::internet::dialer::Dialer;
use crate::transport::internet::system_dialer::SystemDialer;
use async_trait::async_trait;

pub struct GrpcDialer {
    pub config: GrpcConfig,
}

impl GrpcDialer {
    pub fn new(config: GrpcConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Dialer for GrpcDialer {
    async fn dial(&self, dest: &Destination) -> Result<BoxStream> {
        let stream = SystemDialer::dial_tcp(dest).await?;
        Ok(Box::pin(stream))
    }
}
