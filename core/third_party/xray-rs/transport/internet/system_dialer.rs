// Module: transport\internet\system_dialer.rs
// 1:1 Rust implementation corresponding to Go transport\internet\system_dialer.go

use async_trait::async_trait;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::RwLock;

use crate::common::errors::{Error, Result};
use crate::common::net::{Address, BoxStream, Destination, Network};
use crate::transport::internet::sockopt::SocketOptions;

pub type ControllerFunc = Arc<dyn Fn(&str, &str) -> Result<()> + Send + Sync>;

#[async_trait]
pub trait SystemDialerTrait: Send + Sync {
    async fn dial(
        &self,
        src: Option<&Address>,
        dest: &Destination,
        sockopt: Option<&SocketOptions>,
    ) -> Result<BoxStream>;

    fn dest_ip_address(&self) -> Option<IpAddr>;
}

pub struct PacketConnWrapper {
    pub socket: Arc<UdpSocket>,
    pub dest: SocketAddr,
}

impl PacketConnWrapper {
    pub fn new(socket: UdpSocket, dest: SocketAddr) -> Self {
        Self {
            socket: Arc::new(socket),
            dest,
        }
    }

    pub fn remote_addr(&self) -> SocketAddr {
        self.dest
    }
}

impl AsyncRead for PacketConnWrapper {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.socket.poll_recv_from(cx, buf).map(|res| match res {
            Ok(_addr) => Ok(()),
            Err(e) => Err(e),
        })
    }
}

impl AsyncWrite for PacketConnWrapper {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.socket.poll_send_to(cx, buf, self.dest)
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

pub struct DefaultSystemDialer {
    controllers: RwLock<Vec<ControllerFunc>>,
}

impl DefaultSystemDialer {
    pub fn new() -> Self {
        Self {
            controllers: RwLock::new(Vec::new()),
        }
    }

    pub async fn add_controller(&self, ctl: ControllerFunc) {
        let mut list = self.controllers.write().await;
        list.push(ctl);
    }
}

impl Default for DefaultSystemDialer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SystemDialerTrait for DefaultSystemDialer {
    async fn dial(
        &self,
        _src: Option<&Address>,
        dest: &Destination,
        _sockopt: Option<&SocketOptions>,
    ) -> Result<BoxStream> {
        let addr_str = match &dest.address {
            Address::Ipv4(ip) => format!("{}:{}", ip, dest.port),
            Address::Ipv6(ip) => format!("[{}]:{}", ip, dest.port),
            Address::Domain(host) => format!("{}:{}", host, dest.port),
        };

        let addrs: Vec<SocketAddr> = tokio::net::lookup_host(&addr_str)
            .await
            .map_err(Error::Io)?
            .collect();

        if addrs.is_empty() {
            return Err(Error::NotFound(format!("Failed to resolve {}", addr_str)));
        }

        let target_addr = addrs[0];

        // Apply controllers
        {
            let ctls = self.controllers.read().await;
            for ctl in ctls.iter() {
                let net_str = match dest.network {
                    Network::Tcp => "tcp",
                    Network::Udp => "udp",
                };
                let _ = ctl(net_str, &target_addr.to_string());
            }
        }

        if dest.network == Network::Udp {
            #[cfg(target_os = "windows")]
            {
                let iface = crate::proxy::tun::DEFAULT_OUTBOUND_INTERFACE.read().await;
                let socket = crate::proxy::tun::socket_helpers::new_udp_socket(
                    None,
                    iface.as_ref(),
                    Some(target_addr),
                )
                .await
                .map_err(Error::Io)?;
                let wrapper = PacketConnWrapper::new(socket, target_addr);
                return Ok(Box::pin(wrapper));
            }
            #[cfg(not(target_os = "windows"))]
            {
                let bind_addr: SocketAddr = if target_addr.is_ipv4() {
                    "0.0.0.0:0".parse().unwrap()
                } else {
                    "[::]:0".parse().unwrap()
                };
                let socket = UdpSocket::bind(bind_addr).await.map_err(Error::Io)?;
                let wrapper = PacketConnWrapper::new(socket, target_addr);
                return Ok(Box::pin(wrapper));
            }
        }

        #[cfg(target_os = "windows")]
        {
            let iface = crate::proxy::tun::DEFAULT_OUTBOUND_INTERFACE.read().await;
            let stream =
                crate::proxy::tun::socket_helpers::new_tcp_stream(target_addr, iface.as_ref())
                    .await
                    .map_err(Error::Io)?;
            let _ = stream.set_nodelay(true);
            return Ok(Box::pin(stream));
        }

        #[cfg(not(target_os = "windows"))]
        {
            let stream = TcpStream::connect(target_addr).await.map_err(Error::Io)?;
            let _ = stream.set_nodelay(true);
            Ok(Box::pin(stream))
        }
    }

    fn dest_ip_address(&self) -> Option<IpAddr> {
        None
    }
}

pub struct SystemDialer;

impl SystemDialer {
    pub async fn dial_tcp(dest: &Destination) -> Result<TcpStream> {
        let addr_str = match &dest.address {
            Address::Ipv4(ip) => format!("{}:{}", ip, dest.port),
            Address::Ipv6(ip) => format!("[{}]:{}", ip, dest.port),
            Address::Domain(host) => format!("{}:{}", host, dest.port),
        };

        let addrs = tokio::net::lookup_host(&addr_str)
            .await
            .map_err(Error::Io)?;

        #[cfg(target_os = "windows")]
        let iface = crate::proxy::tun::DEFAULT_OUTBOUND_INTERFACE.read().await;

        let mut last_err = None;
        for addr in addrs {
            #[cfg(target_os = "windows")]
            let res = crate::proxy::tun::socket_helpers::new_tcp_stream(addr, iface.as_ref()).await;
            #[cfg(not(target_os = "windows"))]
            let res = TcpStream::connect(addr).await;

            match res {
                Ok(stream) => {
                    let _ = stream.set_nodelay(true);
                    return Ok(stream);
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

    pub async fn dial_addr(addr: SocketAddr) -> Result<TcpStream> {
        #[cfg(target_os = "windows")]
        {
            let iface = crate::proxy::tun::DEFAULT_OUTBOUND_INTERFACE.read().await;
            let stream =
                crate::proxy::tun::socket_helpers::new_tcp_stream(addr, iface.as_ref()).await?;
            let _ = stream.set_nodelay(true);
            Ok(stream)
        }
        #[cfg(not(target_os = "windows"))]
        {
            let stream = TcpStream::connect(addr).await?;
            let _ = stream.set_nodelay(true);
            Ok(stream)
        }
    }

    pub async fn dial(
        src: Option<&Address>,
        dest: &Destination,
        sockopt: Option<&SocketOptions>,
    ) -> Result<BoxStream> {
        let dialer = DefaultSystemDialer::new();
        dialer.dial(src, dest, sockopt).await
    }
}
