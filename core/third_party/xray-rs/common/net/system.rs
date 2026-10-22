use crate::common::errors::{Error, Result};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream, UdpSocket};

pub struct SystemListener;

impl SystemListener {
    pub async fn listen_tcp(addr: SocketAddr) -> Result<TcpListener> {
        TcpListener::bind(addr).await.map_err(Error::Io)
    }

    pub async fn listen_udp(addr: SocketAddr) -> Result<UdpSocket> {
        UdpSocket::bind(addr).await.map_err(Error::Io)
    }

    pub async fn dial_tcp(addr: SocketAddr) -> Result<TcpStream> {
        #[cfg(target_os = "windows")]
        {
            let iface = crate::proxy::tun::DEFAULT_OUTBOUND_INTERFACE.read().await;
            crate::proxy::tun::socket_helpers::new_tcp_stream(addr, iface.as_ref())
                .await
                .map_err(Error::Io)
        }
        #[cfg(not(target_os = "windows"))]
        TcpStream::connect(addr).await.map_err(Error::Io)
    }
}
