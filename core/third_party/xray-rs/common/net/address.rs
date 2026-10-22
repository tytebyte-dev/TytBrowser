// Module: common\net\address.rs
// 1:1 Rust implementation corresponding to Go common\net\address.go

use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::sync::LazyLock;

use crate::common::errors::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum AddressFamily {
    IPv4 = 0,
    IPv6 = 1,
    Domain = 2,
}

impl AddressFamily {
    pub fn is_ipv4(&self) -> bool {
        *self == AddressFamily::IPv4
    }
    pub fn is_ipv6(&self) -> bool {
        *self == AddressFamily::IPv6
    }
    pub fn is_ip(&self) -> bool {
        *self == AddressFamily::IPv4 || *self == AddressFamily::IPv6
    }
    pub fn is_domain(&self) -> bool {
        *self == AddressFamily::Domain
    }
}

pub static LOCAL_HOST_IP: LazyLock<Address> =
    LazyLock::new(|| Address::Ipv4(Ipv4Addr::new(127, 0, 0, 1)));
pub static ANY_IP: LazyLock<Address> = LazyLock::new(|| Address::Ipv4(Ipv4Addr::new(0, 0, 0, 0)));
pub static LOCAL_HOST_DOMAIN: LazyLock<Address> =
    LazyLock::new(|| Address::Domain("localhost".into()));
pub static LOCAL_HOST_IPV6: LazyLock<Address> =
    LazyLock::new(|| Address::Ipv6(Ipv6Addr::LOCALHOST));
pub static ANY_IPV6: LazyLock<Address> = LazyLock::new(|| Address::Ipv6(Ipv6Addr::UNSPECIFIED));

pub fn local_host_ip() -> Address {
    Address::Ipv4(Ipv4Addr::new(127, 0, 0, 1))
}

pub fn any_ip() -> Address {
    Address::Ipv4(Ipv4Addr::new(0, 0, 0, 0))
}

pub fn local_host_domain() -> Address {
    Address::Domain("localhost".into())
}

pub fn local_host_ipv6() -> Address {
    Address::Ipv6(Ipv6Addr::LOCALHOST)
}

pub fn any_ipv6() -> Address {
    Address::Ipv6(Ipv6Addr::UNSPECIFIED)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Address {
    Ipv4(Ipv4Addr),
    Ipv6(Ipv6Addr),
    Domain(String),
}

impl Address {
    pub fn ip(ip: IpAddr) -> Self {
        match ip {
            IpAddr::V4(v4) => Address::Ipv4(v4),
            IpAddr::V6(v6) => {
                if let Some(v4) = v6.to_ipv4_mapped() {
                    Address::Ipv4(v4)
                } else {
                    Address::Ipv6(v6)
                }
            }
        }
    }

    pub fn ipv4(v4: Ipv4Addr) -> Self {
        Address::Ipv4(v4)
    }

    pub fn ipv6(v6: Ipv6Addr) -> Self {
        if let Some(v4) = v6.to_ipv4_mapped() {
            Address::Ipv4(v4)
        } else {
            Address::Ipv6(v6)
        }
    }

    pub fn domain(domain: impl Into<String>) -> Self {
        Address::Domain(domain.into())
    }

    pub fn is_ip(&self) -> bool {
        matches!(self, Address::Ipv4(_) | Address::Ipv6(_))
    }

    pub fn is_ipv4(&self) -> bool {
        matches!(self, Address::Ipv4(_))
    }

    pub fn is_ipv6(&self) -> bool {
        matches!(self, Address::Ipv6(_))
    }

    pub fn is_domain(&self) -> bool {
        matches!(self, Address::Domain(_))
    }

    pub fn family(&self) -> AddressFamily {
        match self {
            Address::Ipv4(_) => AddressFamily::IPv4,
            Address::Ipv6(_) => AddressFamily::IPv6,
            Address::Domain(_) => AddressFamily::Domain,
        }
    }

    pub fn to_ip(&self) -> Option<IpAddr> {
        match self {
            Address::Ipv4(ip) => Some(IpAddr::V4(*ip)),
            Address::Ipv6(ip) => Some(IpAddr::V6(*ip)),
            Address::Domain(_) => None,
        }
    }

    pub fn as_ip(&self) -> Option<IpAddr> {
        self.to_ip()
    }

    pub fn ip_bytes(&self) -> Option<Vec<u8>> {
        match self {
            Address::Ipv4(v4) => Some(v4.octets().to_vec()),
            Address::Ipv6(v6) => Some(v6.octets().to_vec()),
            Address::Domain(_) => None,
        }
    }

    pub fn domain_name(&self) -> Option<&str> {
        match self {
            Address::Domain(d) => Some(d.as_str()),
            _ => None,
        }
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Ipv4(ip) => write!(f, "{}", ip),
            Address::Ipv6(ip) => write!(f, "[{}]", ip),
            Address::Domain(domain) => write!(f, "{}", domain),
        }
    }
}

impl FromStr for Address {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let trimmed = s.trim();
        let cleaned = if trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed.len() >= 2 {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        };

        if let Ok(ip) = cleaned.parse::<IpAddr>() {
            return match ip {
                IpAddr::V4(v4) => Ok(Address::Ipv4(v4)),
                IpAddr::V6(v6) => {
                    if let Some(v4) = v6.to_ipv4_mapped() {
                        Ok(Address::Ipv4(v4))
                    } else {
                        Ok(Address::Ipv6(v6))
                    }
                }
            };
        }

        if !trimmed.is_empty() {
            return Ok(Address::Domain(trimmed.to_string()));
        }
        Err(Error::Protocol("Empty address string".into()))
    }
}

impl From<IpAddr> for Address {
    fn from(ip: IpAddr) -> Self {
        Address::ip(ip)
    }
}

impl From<Ipv4Addr> for Address {
    fn from(v4: Ipv4Addr) -> Self {
        Address::Ipv4(v4)
    }
}

impl From<Ipv6Addr> for Address {
    fn from(v6: Ipv6Addr) -> Self {
        Address::ipv6(v6)
    }
}

pub fn parse_address(addr: &str) -> Address {
    addr.parse::<Address>()
        .unwrap_or_else(|_| Address::Domain(addr.to_string()))
}

pub fn ip_address(ip: impl Into<IpAddr>) -> Address {
    Address::ip(ip.into())
}

pub fn ip_address_from_bytes(bytes: &[u8]) -> Option<Address> {
    match bytes.len() {
        4 => {
            let mut octets = [0u8; 4];
            octets.copy_from_slice(bytes);
            Some(Address::Ipv4(Ipv4Addr::from(octets)))
        }
        16 => {
            if bytes[..10] == [0u8; 10] && bytes[10] == 0xff && bytes[11] == 0xff {
                let mut octets = [0u8; 4];
                octets.copy_from_slice(&bytes[12..16]);
                Some(Address::Ipv4(Ipv4Addr::from(octets)))
            } else {
                let mut octets = [0u8; 16];
                octets.copy_from_slice(bytes);
                Some(Address::Ipv6(Ipv6Addr::from(octets)))
            }
        }
        _ => None,
    }
}

pub fn domain_address(domain: impl Into<String>) -> Address {
    Address::Domain(domain.into())
}
