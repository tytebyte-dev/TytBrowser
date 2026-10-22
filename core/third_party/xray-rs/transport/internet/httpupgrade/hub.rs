// Module: transport\internet\httpupgrade\hub.rs
// 1:1 Rust implementation corresponding to Go transport\internet\httpupgrade\hub.go

use super::config::HttpUpgradeConfig;
use super::connection::HttpUpgradeConnection;
use super::httpupgrade::HttpUpgradeStream;
use crate::common::errors::Result;
use crate::transport::internet::tcp::TcpHub;
use std::net::SocketAddr;

pub struct HttpUpgradeHub {
    tcp_hub: TcpHub,
    config: HttpUpgradeConfig,
}

impl HttpUpgradeHub {
    pub async fn listen(addr: SocketAddr, config: HttpUpgradeConfig) -> Result<Self> {
        let tcp_hub = TcpHub::listen(addr).await?;
        Ok(Self { tcp_hub, config })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.tcp_hub.local_addr()
    }

    pub async fn accept(&self) -> Result<HttpUpgradeConnection> {
        let (raw_stream, remote_addr) = self.tcp_hub.accept().await?;
        let expected_host = if self.config.host.is_empty() {
            None
        } else {
            Some(self.config.host.as_str())
        };
        let expected_path = Some(self.config.get_normalized_path());
        let upgrade_stream = HttpUpgradeStream::server_handshake(
            raw_stream,
            expected_host,
            expected_path.as_deref(),
        )
        .await?;
        Ok(HttpUpgradeConnection::new(
            upgrade_stream,
            remote_addr.to_string(),
        ))
    }
}
