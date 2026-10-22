// Module: app\dns\nameserver_udp.rs
// 1:1 Rust implementation corresponding to Go app\dns\nameserver_udp.go

use async_trait::async_trait;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
#[allow(unused_imports)]
use tokio::net::UdpSocket;

use super::nameserver::{NameServer, build_dns_query_typed, parse_dns_response};
use crate::common::errors::{Error, Result};

pub struct UdpNameServer {
    pub server_addr: SocketAddr,
    pub timeout: Duration,
}

impl UdpNameServer {
    pub fn new(server_addr: SocketAddr) -> Self {
        Self {
            server_addr,
            timeout: Duration::from_secs(5),
        }
    }

    pub fn with_timeout(server_addr: SocketAddr, timeout: Duration) -> Self {
        Self {
            server_addr,
            timeout,
        }
    }

    pub fn server_addr(&self) -> SocketAddr {
        self.server_addr
    }
}

#[async_trait]
impl NameServer for UdpNameServer {
    fn name(&self) -> &str {
        "udp"
    }

    async fn query_ip(&self, domain: &str) -> Result<Vec<IpAddr>> {
        let clean = domain.trim_end_matches('.');
        let _bind_addr = if self.server_addr.is_ipv6() {
            "[::]:0"
        } else {
            "0.0.0.0:0"
        };

        #[cfg(target_os = "windows")]
        let socket = {
            let iface = crate::proxy::tun::DEFAULT_OUTBOUND_INTERFACE.read().await;
            let sock = crate::proxy::tun::socket_helpers::new_udp_socket(
                None,
                iface.as_ref(),
                Some(self.server_addr),
            )
            .await
            .map_err(Error::Io)?;
            sock.connect(self.server_addr).await.map_err(Error::Io)?;
            sock
        };
        #[cfg(not(target_os = "windows"))]
        let socket = {
            let socket = UdpSocket::bind(bind_addr).await.map_err(Error::Io)?;
            socket.connect(self.server_addr).await.map_err(Error::Io)?;
            socket
        };

        let query_a = build_dns_query_typed(clean, 1);
        socket.send(&query_a).await.map_err(Error::Io)?;

        let mut buf = vec![0u8; 2048];
        let n = tokio::time::timeout(self.timeout, socket.recv(&mut buf))
            .await
            .map_err(|_| Error::Timeout)?
            .map_err(Error::Io)?;

        parse_dns_response(&buf[..n], clean)
    }
}
