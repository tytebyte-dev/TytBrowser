// Module: proxy\wireguard\bind.rs
// 1:1 Rust implementation corresponding to Go proxy\wireguard\bind.go

use crate::common::errors::Result;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub struct WireGuardBind {
    socket: UdpSocket,
}

impl WireGuardBind {
    pub async fn bind(addr: SocketAddr) -> Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        Ok(Self { socket })
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.socket.local_addr()?)
    }
}
