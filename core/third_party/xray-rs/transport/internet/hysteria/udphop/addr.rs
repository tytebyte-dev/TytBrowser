// Module: transport\internet\hysteria\udphop\addr.rs
// 1:1 Rust implementation corresponding to Go transport\internet\hysteria\udphop\addr.go

use std::net::{IpAddr, SocketAddr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UDPHopAddr {
    pub ip: IpAddr,
    pub ports: Vec<u16>,
    pub port_str: String,
}

impl UDPHopAddr {
    pub fn new(ip: IpAddr, ports: Vec<u16>, port_str: impl Into<String>) -> Self {
        Self {
            ip,
            ports,
            port_str: port_str.into(),
        }
    }

    pub fn network(&self) -> &'static str {
        "udphop"
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.ip, self.port_str)
    }

    pub fn addrs(&self) -> Vec<SocketAddr> {
        self.ports
            .iter()
            .map(|&p| SocketAddr::new(self.ip, p))
            .collect()
    }
}
