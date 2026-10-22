use super::{Address, Network};
use crate::common::errors::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::str::FromStr;

/// Represents a network destination (Address + Port + Network).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Destination {
    pub address: Address,
    pub port: u16,
    #[serde(default)]
    pub network: Network,
}

impl Destination {
    pub fn new(address: Address, port: u16) -> Self {
        Self {
            address,
            port,
            network: Network::Tcp,
        }
    }

    pub fn tcp(address: Address, port: u16) -> Self {
        Self {
            address,
            port,
            network: Network::Tcp,
        }
    }

    pub fn udp(address: Address, port: u16) -> Self {
        Self {
            address,
            port,
            network: Network::Udp,
        }
    }

    pub fn from_ip_port(ip: IpAddr, port: u16) -> Self {
        Self::new(Address::from(ip), port)
    }

    pub fn from_domain_port(domain: impl Into<String>, port: u16) -> Self {
        Self::new(Address::Domain(domain.into()), port)
    }

    pub fn to_socket_addr(&self) -> Option<SocketAddr> {
        self.address
            .to_ip()
            .map(|ip| SocketAddr::new(ip, self.port))
    }

    pub fn net_addr(&self) -> String {
        match &self.address {
            Address::Ipv4(ip) => format!("{}:{}", ip, self.port),
            Address::Ipv6(ip) => format!("[{}]:{}", ip, self.port),
            Address::Domain(domain) => format!("{}:{}", domain, self.port),
        }
    }

    pub fn is_ipv6(&self) -> bool {
        self.address.is_ipv6()
    }

    pub fn parse_str(s: &str, network: Network) -> Result<Self> {
        let s = s.trim();
        if let Some((host, port_str)) = s.rsplit_once(':') {
            let port = port_str
                .parse::<u16>()
                .map_err(|e| Error::Config(format!("Invalid port: {}", e)))?;
            let host = host.trim_matches('[').trim_matches(']');
            if let Ok(ip) = IpAddr::from_str(host) {
                let mut d = Destination::from_ip_port(ip, port);
                d.network = network;
                Ok(d)
            } else {
                let mut d = Destination::from_domain_port(host, port);
                d.network = network;
                Ok(d)
            }
        } else if let Ok(port) = s.parse::<u16>() {
            let mut d = Destination::from_domain_port("127.0.0.1", port);
            d.network = network;
            Ok(d)
        } else {
            Err(Error::Config(format!("Invalid destination string: {}", s)))
        }
    }
}

impl fmt::Display for Destination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.address {
            Address::Ipv4(ip) => write!(f, "{}:{}", ip, self.port),
            Address::Ipv6(ip) => write!(f, "[{}]:{}", ip, self.port),
            Address::Domain(domain) => write!(f, "{}:{}", domain, self.port),
        }
    }
}

impl Default for Destination {
    fn default() -> Self {
        Self {
            address: Address::Ipv4(std::net::Ipv4Addr::UNSPECIFIED),
            port: 0,
            network: Network::Udp,
        }
    }
}

impl From<SocketAddr> for Destination {
    fn from(addr: SocketAddr) -> Self {
        Self::new(Address::from(addr.ip()), addr.port())
    }
}

impl PartialEq<SocketAddr> for Destination {
    fn eq(&self, other: &SocketAddr) -> bool {
        if self.port != other.port() {
            return false;
        }
        self.to_socket_addr() == Some(*other)
    }
}

impl PartialEq<Destination> for SocketAddr {
    fn eq(&self, other: &Destination) -> bool {
        other == self
    }
}

impl FromStr for Destination {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();
        if let Some(rest) = s.strip_prefix('[')
            && let Some((ip_part, port_part)) = rest.split_once("]:")
        {
            let ip = Ipv6Addr::from_str(ip_part)
                .map_err(|e| Error::AddressParse(format!("Invalid IPv6 in {}: {}", s, e)))?;
            let port = port_part
                .parse::<u16>()
                .map_err(|e| Error::AddressParse(format!("Invalid port in {}: {}", s, e)))?;
            return Ok(Destination::new(Address::Ipv6(ip), port));
        }

        if let Some((host_part, port_part)) = s.rsplit_once(':') {
            let port = port_part
                .parse::<u16>()
                .map_err(|e| Error::AddressParse(format!("Invalid port in {}: {}", s, e)))?;
            let address = Address::from_str(host_part)?;
            return Ok(Destination::new(address, port));
        }

        Err(Error::AddressParse(format!(
            "Missing port in destination address: {}",
            s
        )))
    }
}

pub fn tcp_destination(address: Address, port: u16) -> Destination {
    Destination::tcp(address, port)
}

pub fn udp_destination(address: Address, port: u16) -> Destination {
    Destination::udp(address, port)
}

pub fn parse_destination(dest: &str) -> Result<Destination> {
    let (network, rest) = if let Some(stripped) = dest.strip_prefix("tcp:") {
        (Network::Tcp, stripped)
    } else if let Some(stripped) = dest.strip_prefix("udp:") {
        (Network::Udp, stripped)
    } else {
        (Network::Tcp, dest)
    };
    Destination::parse_str(rest, network)
}
