// Module: transport\internet\grpc\hub.rs
// 1:1 Rust implementation corresponding to Go transport\internet\grpc\hub.go

use super::config::GrpcConfig;
use crate::common::errors::Result;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub struct GrpcListener {
    _config: GrpcConfig,
    listener: TcpListener,
}

impl GrpcListener {
    pub async fn bind(addr: SocketAddr, config: GrpcConfig) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self {
            _config: config,
            listener,
        })
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.listener.local_addr()?)
    }

    pub async fn accept(&self) -> Result<(crate::common::net::BoxStream, SocketAddr)> {
        let (stream, addr) = self.listener.accept().await?;
        Ok((Box::pin(stream), addr))
    }
}
