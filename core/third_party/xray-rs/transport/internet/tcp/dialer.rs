// Module: transport\internet\tcp\dialer.rs
// 1:1 Rust implementation corresponding to Go transport\internet\tcp\dialer.go

use std::net::SocketAddr;
#[allow(unused_imports)]
use tokio::net::TcpStream as TokioTcpStream;

use crate::common::errors::{Error, Result};
use crate::common::net::{BoxStream, Destination};

pub struct TcpDialer;

impl TcpDialer {
    pub async fn dial(dest: &Destination) -> Result<BoxStream> {
        #[cfg(target_os = "windows")]
        {
            let iface = crate::proxy::tun::DEFAULT_OUTBOUND_INTERFACE.read().await;
            match &dest.address {
                crate::common::net::Address::Ipv4(ip) => {
                    let addr = SocketAddr::new(std::net::IpAddr::V4(*ip), dest.port);
                    let stream =
                        crate::proxy::tun::socket_helpers::new_tcp_stream(addr, iface.as_ref())
                            .await
                            .map_err(Error::Io)?;
                    let _ = stream.set_nodelay(true);
                    Ok(Box::pin(stream))
                }
                crate::common::net::Address::Ipv6(ip) => {
                    let addr = SocketAddr::new(std::net::IpAddr::V6(*ip), dest.port);
                    let stream =
                        crate::proxy::tun::socket_helpers::new_tcp_stream(addr, iface.as_ref())
                            .await
                            .map_err(Error::Io)?;
                    let _ = stream.set_nodelay(true);
                    Ok(Box::pin(stream))
                }
                crate::common::net::Address::Domain(domain) => {
                    let addr_str = format!("{}:{}", domain, dest.port);
                    let addrs = tokio::net::lookup_host(&addr_str)
                        .await
                        .map_err(Error::Io)?;
                    let mut last_err = None;
                    for addr in addrs {
                        match crate::proxy::tun::socket_helpers::new_tcp_stream(
                            addr,
                            iface.as_ref(),
                        )
                        .await
                        {
                            Ok(stream) => {
                                let _ = stream.set_nodelay(true);
                                return Ok(Box::pin(stream));
                            }
                            Err(e) => {
                                last_err = Some(e);
                            }
                        }
                    }
                    Err(Error::Io(last_err.unwrap_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!("Failed to resolve or connect to {}", addr_str),
                        )
                    })))
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let stream = match &dest.address {
                crate::common::net::Address::Ipv4(ip) => {
                    let addr = SocketAddr::new(std::net::IpAddr::V4(*ip), dest.port);
                    TokioTcpStream::connect(addr).await?
                }
                crate::common::net::Address::Ipv6(ip) => {
                    let addr = SocketAddr::new(std::net::IpAddr::V6(*ip), dest.port);
                    TokioTcpStream::connect(addr).await?
                }
                crate::common::net::Address::Domain(domain) => {
                    let addr_str = format!("{}:{}", domain, dest.port);
                    TokioTcpStream::connect(&addr_str).await.map_err(|e| {
                        Error::Io(std::io::Error::new(
                            e.kind(),
                            format!("Failed to connect to {}: {}", addr_str, e),
                        ))
                    })?
                }
            };

            let _ = stream.set_nodelay(true);
            Ok(Box::pin(stream))
        }
    }
}
