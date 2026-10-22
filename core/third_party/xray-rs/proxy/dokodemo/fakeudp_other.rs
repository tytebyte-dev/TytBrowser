// Module: proxy\dokodemo\fakeudp_other.rs
// 1:1 Rust implementation corresponding to Go proxy\dokodemo\fakeudp_other.go

use crate::common::errors::{Error, Result};
use std::net::SocketAddr;

pub fn fake_udp_other(_addr: SocketAddr, _mark: u32) -> Result<tokio::net::UdpSocket> {
    Err(Error::Unsupported(
        "fake UDP is only supported on Linux".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fake_udp_other_unsupported() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let res = fake_udp_other(addr, 0);
        assert!(res.is_err());
    }
}
