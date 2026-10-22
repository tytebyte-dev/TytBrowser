// Module: transport\internet\udp\hub.rs
// 1:1 Rust implementation corresponding to Go transport\internet\udp\hub.go

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use crate::common::buf::Buffer;
use crate::common::errors::{Error, Result};
use crate::common::net::{Address, Destination, Port};
pub use crate::common::protocol::udp::UdpPacket;
use crate::transport::internet::MemoryStreamConfig;

#[cfg(target_os = "macos")]
use super::hub_darwin as platform_hub;
#[cfg(target_os = "freebsd")]
use super::hub_freebsd as platform_hub;
#[cfg(target_os = "linux")]
use super::hub_linux as platform_hub;
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "freebsd")))]
use super::hub_other as platform_hub;

#[derive(Debug, Clone, Copy)]
pub enum HubOption {
    Capacity(usize),
    ReceiveOriginalDestination(bool),
}

pub struct Hub {
    socket: Arc<UdpSocket>,
    cache_rx: mpsc::Receiver<UdpPacket>,
    capacity: usize,
    recv_orig_dest: bool,
    closed: Arc<AtomicBool>,
}

pub type UdpHub = Hub;

impl Hub {
    pub async fn listen_udp(
        address: &Address,
        port: Port,
        stream_settings: Option<&MemoryStreamConfig>,
        options: &[HubOption],
    ) -> Result<Self> {
        let mut capacity = 256;
        let mut recv_orig_dest = false;

        for opt in options {
            match opt {
                HubOption::Capacity(c) => capacity = *c,
                HubOption::ReceiveOriginalDestination(r) => recv_orig_dest = *r,
            }
        }

        let bind_addr = if let Some(domain) = address.domain_name() {
            if domain == "localhost" {
                Address::ip([127, 0, 0, 1].into())
            } else {
                return Err(Error::Other(format!(
                    "domain address is not allowed for listening: {}",
                    domain
                )));
            }
        } else {
            address.clone()
        };

        if let Some(ss) = stream_settings
            && let Some(sockopt) = &ss.socket_settings
            && sockopt.receive_original_dest_address
        {
            recv_orig_dest = true;
        }

        let ip = bind_addr
            .to_ip()
            .ok_or_else(|| Error::Other("invalid IP address".into()))?;
        let sock_addr = SocketAddr::new(ip, port.value());
        let socket = Arc::new(UdpSocket::bind(sock_addr).await.map_err(Error::Io)?);

        let (cache_tx, cache_rx) = mpsc::channel(capacity);
        let closed = Arc::new(AtomicBool::new(false));

        let s_clone = socket.clone();
        let closed_clone = closed.clone();

        tokio::spawn(async move {
            let mut payload = vec![0u8; 65535];
            let mut oob = vec![0u8; 256];

            while !closed_clone.load(Ordering::Relaxed) {
                match platform_hub::read_udp_msg(&s_clone, &mut payload, &mut oob).await {
                    Ok((n, noob, _, Some(addr))) => {
                        if n == 0 {
                            continue;
                        }
                        let buffer = Buffer::from_bytes(&payload[..n]);
                        let source = Destination::udp(Address::from(addr.ip()), addr.port());
                        let target = if recv_orig_dest && noob > 0 {
                            platform_hub::retrieve_original_dest(&oob[..noob]).unwrap_or_default()
                        } else {
                            Destination::default()
                        };

                        let packet = UdpPacket::new(buffer, source, target);
                        if cache_tx.send(packet).await.is_err() {
                            break;
                        }
                    }
                    Ok((_, _, _, None)) => {}
                    Err(_) => {
                        if closed_clone.load(Ordering::Relaxed) {
                            break;
                        }
                    }
                }
            }
        });

        Ok(Self {
            socket,
            cache_rx,
            capacity,
            recv_orig_dest,
            closed,
        })
    }

    pub async fn bind(addr: SocketAddr) -> Result<Self> {
        let address = Address::from(addr.ip());
        let port = Port::new(addr.port());
        Self::listen_udp(&address, port, None, &[]).await
    }

    pub async fn recv(&mut self) -> Option<UdpPacket> {
        self.cache_rx.recv().await
    }

    pub async fn write_to(&self, payload: &[u8], dest: &Destination) -> Result<usize> {
        if let Some(target) = dest.to_socket_addr() {
            self.socket
                .send_to(payload, target)
                .await
                .map_err(Error::Io)
        } else {
            Err(Error::Other(format!(
                "cannot resolve destination: {:?}",
                dest
            )))
        }
    }

    pub async fn send_to(&self, payload: &[u8], target: SocketAddr) -> Result<usize> {
        self.socket
            .send_to(payload, target)
            .await
            .map_err(Error::Io)
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.socket.local_addr().map_err(Error::Io)
    }

    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn receive_original_destination(&self) -> bool {
        self.recv_orig_dest
    }
}
