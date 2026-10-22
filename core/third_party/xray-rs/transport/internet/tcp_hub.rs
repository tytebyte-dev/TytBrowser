// Module: transport\internet\tcp_hub.rs
// 1:1 Rust implementation corresponding to Go transport\internet\tcp_hub.go

use crate::common::errors::{Error, Result};
use crate::common::net::Address;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, UdpSocket};

pub use super::tcp::TcpHub;

type ListenCallback = Arc<dyn Fn(SocketAddr) -> Result<TcpListener> + Send + Sync>;

lazy_static::lazy_static! {
    static ref TRANSPORT_LISTENERS: Mutex<HashMap<String, ListenCallback>> = Mutex::new(HashMap::new());
}

pub fn register_transport_listener<F>(protocol: &str, listener: F) -> Result<()>
where
    F: Fn(SocketAddr) -> Result<TcpListener> + Send + Sync + 'static,
{
    let mut map = TRANSPORT_LISTENERS.lock().unwrap();
    if map.contains_key(protocol) {
        return Err(Error::Protocol(format!(
            "{} listener already registered",
            protocol
        )));
    }
    map.insert(protocol.to_string(), Arc::new(listener));
    Ok(())
}

pub async fn listen_tcp(address: &Address, port: u16) -> Result<TcpListener> {
    let addr = match address {
        Address::Ipv4(ip) => SocketAddr::new(std::net::IpAddr::V4(*ip), port),
        Address::Ipv6(ip) => SocketAddr::new(std::net::IpAddr::V6(*ip), port),
        Address::Domain(domain) => {
            if domain == "localhost" {
                SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), port)
            } else {
                return Err(Error::Protocol(format!(
                    "Domain address not allowed for listening: {}",
                    domain
                )));
            }
        }
    };
    TcpListener::bind(addr).await.map_err(Into::into)
}

pub async fn listen_system(addr: SocketAddr) -> Result<TcpListener> {
    TcpListener::bind(addr).await.map_err(Into::into)
}

pub async fn listen_system_packet(addr: SocketAddr) -> Result<UdpSocket> {
    UdpSocket::bind(addr).await.map_err(Into::into)
}
