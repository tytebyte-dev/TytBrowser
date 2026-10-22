// Module: transport\internet\udp\dialer.rs
// 1:1 Rust implementation corresponding to Go transport\internet\udp\dialer.go

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

use crate::common::errors::{Error, Result};
use crate::common::net::Destination;
use crate::transport::internet::MemoryStreamConfig;

pub struct UdpDialer;

impl UdpDialer {
    pub async fn dial(
        dest: &Destination,
        _stream_settings: Option<&MemoryStreamConfig>,
    ) -> Result<Arc<UdpSocket>> {
        let local_addr: SocketAddr = if dest.is_ipv6() {
            "[::]:0".parse().unwrap()
        } else {
            "0.0.0.0:0".parse().unwrap()
        };
        let socket = UdpSocket::bind(local_addr).await.map_err(Error::Io)?;

        if let Some(target_addr) = dest.to_socket_addr() {
            socket.connect(target_addr).await.map_err(Error::Io)?;
        }
        Ok(Arc::new(socket))
    }
}
