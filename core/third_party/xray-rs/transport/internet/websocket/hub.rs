// Module: transport\internet\websocket\hub.rs
// 1:1 Rust implementation corresponding to Go transport\internet\websocket\hub.go

use super::ws::WebSocketStream;
use crate::common::errors::Result;
use crate::common::net::BoxStream;
use crate::transport::internet::tcp::TcpHub;
use std::net::SocketAddr;

pub struct WebSocketHub {
    tcp_hub: TcpHub,
}

impl WebSocketHub {
    pub async fn listen(addr: SocketAddr) -> Result<Self> {
        let tcp_hub = TcpHub::listen(addr).await?;
        Ok(Self { tcp_hub })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.tcp_hub.local_addr()
    }

    pub async fn accept(&self) -> Result<(BoxStream, SocketAddr)> {
        let (raw_stream, remote_addr) = self.tcp_hub.accept().await?;
        let ws_stream = WebSocketStream::server_handshake(raw_stream).await?;
        Ok((ws_stream, remote_addr))
    }
}
