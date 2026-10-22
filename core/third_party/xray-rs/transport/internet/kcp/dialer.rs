// Module: transport\internet\kcp\dialer.rs
// 1:1 Rust implementation corresponding to Go transport\internet\kcp\dialer.go

use super::connection::KcpConnection;
use crate::common::errors::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

pub struct KcpDialer;

impl KcpDialer {
    pub async fn dial(addr: SocketAddr, conv: u16) -> Result<KcpConnection> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(addr).await?;
        Ok(KcpConnection::new(conv, Arc::new(socket)))
    }
}
