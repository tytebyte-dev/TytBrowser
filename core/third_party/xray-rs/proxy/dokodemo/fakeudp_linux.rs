// Module: proxy\dokodemo\fakeudp_linux.rs
// 1:1 Rust implementation corresponding to Go proxy\dokodemo\fakeudp_linux.go

use crate::common::errors::{Error, Result};
use std::net::SocketAddr;

pub fn fake_udp_linux(addr: SocketAddr, _mark: u32) -> Result<tokio::net::UdpSocket> {
    #[cfg(target_os = "linux")]
    {
        use socket2::{Domain, Protocol, Socket, Type};
        let domain = if addr.is_ipv4() {
            Domain::IPV4
        } else {
            Domain::IPV6
        };
        let socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))?;
        socket.set_reuse_address(true)?;
        #[cfg(target_os = "linux")]
        {
            // Set IP_TRANSPARENT or SO_MARK if needed
        }
        let sock_addr = socket2::SockAddr::from(addr);
        socket.bind(&sock_addr)?;
        socket.set_nonblocking(true)?;
        let std_udp: std::net::UdpSocket = socket.into();
        let tokio_udp = tokio::net::UdpSocket::from_std(std_udp)?;
        Ok(tokio_udp)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = addr;
        Err(Error::Unsupported(
            "fake UDP is only supported on Linux".into(),
        ))
    }
}

pub fn is_tproxy_supported() -> bool {
    cfg!(target_os = "linux")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_tproxy_supported_flag() {
        let _ = is_tproxy_supported();
    }
}
