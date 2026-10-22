// Module: proxy\wireguard\client.rs
// 1:1 Rust implementation corresponding to Go proxy\wireguard\client.go

use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UdpSocket;

use super::config::WireGuardConfig;
use crate::common::errors::{Error, Result};
use crate::common::net::BoxStream;
use crate::common::protocol::SessionContext;
use crate::features::outbound::OutboundHandler;

pub struct Client {
    tag: String,
    config: WireGuardConfig,
}

impl Client {
    pub fn new(tag: impl Into<String>, config: WireGuardConfig) -> Self {
        Self {
            tag: tag.into(),
            config,
        }
    }

    pub fn config(&self) -> &WireGuardConfig {
        &self.config
    }
}

#[async_trait]
impl OutboundHandler for Client {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn connect(&self, _session: &SessionContext) -> Result<BoxStream> {
        let peer = self
            .config
            .peers
            .first()
            .ok_or_else(|| Error::Config("No WireGuard peers defined".into()))?;

        let endpoint_addr: SocketAddr = peer.endpoint.parse().map_err(|e| {
            Error::AddressParse(format!(
                "Invalid WireGuard peer endpoint {}: {}",
                peer.endpoint, e
            ))
        })?;

        let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
        socket.connect(endpoint_addr).await?;

        let (client_stream, mut server_stream) = tokio::io::duplex(64 * 1024);
        let socket_recv = socket.clone();
        let socket_send = socket;

        tokio::spawn(async move {
            let mut stream_buf = vec![0u8; 64 * 1024];
            let mut udp_buf = vec![0u8; 64 * 1024];
            loop {
                tokio::select! {
                    res = server_stream.read(&mut stream_buf) => {
                        match res {
                            Ok(0) => break,
                            Ok(n) => {
                                if socket_send.send(&stream_buf[..n]).await.is_err() {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    res = socket_recv.recv(&mut udp_buf) => {
                        match res {
                            Ok(n) => {
                                if server_stream.write_all(&udp_buf[..n]).await.is_err() {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                }
            }
        });

        Ok(Box::pin(client_stream))
    }
}
