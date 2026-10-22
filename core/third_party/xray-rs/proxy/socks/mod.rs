pub mod client;
pub mod protocol;
pub mod server;

use crate::common::errors::{Error, Result};
use crate::common::net::{Address, BoxStream, Destination, Network};
use crate::common::protocol::SessionContext;
use crate::features::inbound::{InboundHandler, InboundResult};
use async_trait::async_trait;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub use client::Client;
pub use protocol::SocksProtocol;

pub struct Server {
    tag: String,
}

impl Server {
    pub fn new(tag: impl Into<String>) -> Self {
        Self { tag: tag.into() }
    }
}

#[async_trait]
impl InboundHandler for Server {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn handle_connection(
        &self,
        mut stream: BoxStream,
        remote_addr: SocketAddr,
    ) -> Result<InboundResult> {
        let ver = stream.read_u8().await.map_err(Error::Io)?;
        if ver != 0x05 {
            return Err(Error::Protocol(format!(
                "Unsupported SOCKS version: {}",
                ver
            )));
        }

        let nmethods = stream.read_u8().await.map_err(Error::Io)? as usize;
        let mut methods = vec![0u8; nmethods];
        stream.read_exact(&mut methods).await.map_err(Error::Io)?;

        stream.write_all(&[0x05, 0x00]).await.map_err(Error::Io)?;
        stream.flush().await.map_err(Error::Io)?;

        let ver = stream.read_u8().await.map_err(Error::Io)?;
        if ver != 0x05 {
            return Err(Error::Protocol(format!(
                "Invalid SOCKS5 version in request: {}",
                ver
            )));
        }

        let cmd = stream.read_u8().await.map_err(Error::Io)?;
        let _rsv = stream.read_u8().await.map_err(Error::Io)?;
        let atyp = stream.read_u8().await.map_err(Error::Io)?;

        let address = match atyp {
            0x01 => {
                let mut ip_bytes = [0u8; 4];
                stream.read_exact(&mut ip_bytes).await.map_err(Error::Io)?;
                Address::Ipv4(Ipv4Addr::from(ip_bytes))
            }
            0x03 => {
                let len = stream.read_u8().await.map_err(Error::Io)? as usize;
                let mut domain_bytes = vec![0u8; len];
                stream
                    .read_exact(&mut domain_bytes)
                    .await
                    .map_err(Error::Io)?;
                let domain = String::from_utf8(domain_bytes)
                    .map_err(|e| Error::Protocol(format!("Invalid domain string: {}", e)))?;
                Address::Domain(domain)
            }
            0x04 => {
                let mut ip_bytes = [0u8; 16];
                stream.read_exact(&mut ip_bytes).await.map_err(Error::Io)?;
                Address::Ipv6(Ipv6Addr::from(ip_bytes))
            }
            _ => {
                return Err(Error::Protocol(format!(
                    "Unsupported SOCKS5 ATYP: {}",
                    atyp
                )));
            }
        };

        let port = stream.read_u16().await.map_err(Error::Io)?;

        let network = match cmd {
            0x01 => Network::Tcp,
            0x03 => Network::Udp,
            _ => {
                return Err(Error::Protocol(format!(
                    "Unsupported SOCKS5 command: {}",
                    cmd
                )));
            }
        };

        stream
            .write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
            .await
            .map_err(Error::Io)?;
        stream.flush().await.map_err(Error::Io)?;

        let destination = Destination {
            network,
            address,
            port,
        };

        let mut session = SessionContext::new(&self.tag, destination);
        session.source = Some(remote_addr);

        Ok(InboundResult { stream, session })
    }
}
