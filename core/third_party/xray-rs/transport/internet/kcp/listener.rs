// Module: transport\internet\kcp\listener.rs
// 1:1 Rust implementation corresponding to Go transport\internet\kcp\listener.go

use super::connection::KcpConnection;
use crate::common::errors::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;

pub struct KcpListener {
    _socket: Arc<UdpSocket>,
    sessions: Arc<RwLock<HashMap<u16, KcpConnection>>>,
}

impl KcpListener {
    pub async fn bind(addr: SocketAddr) -> Result<Self> {
        let socket = Arc::new(UdpSocket::bind(addr).await?);
        Ok(Self {
            _socket: socket,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn get_session(&self, conv: u16) -> Option<KcpConnection> {
        let guard = self.sessions.read().await;
        guard.get(&conv).cloned()
    }
}
