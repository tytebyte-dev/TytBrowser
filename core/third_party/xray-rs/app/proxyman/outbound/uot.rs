use crate::common::errors::{Error, Result};
use crate::common::net::{Address, Destination};

pub const UOT_MAGIC_ADDRESS: &str = "sp-uot.v2fly.org";
pub const UOT_LEGACY_MAGIC_ADDRESS: &str = "sp-uot-v1.v2fly.org";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UotVersion {
    Legacy = 1,
    Standard = 2,
}

pub struct UotPacket {
    pub destination: Destination,
    pub payload: Vec<u8>,
}

impl UotPacket {
    pub fn new(destination: Destination, payload: Vec<u8>) -> Self {
        Self {
            destination,
            payload,
        }
    }

    pub fn encode(&self, version: UotVersion) -> Vec<u8> {
        let mut buf = Vec::new();
        match version {
            UotVersion::Standard => {
                buf.push(0x00); // Command: Connect/Data
                match &self.destination.address {
                    Address::Ipv4(ip) => {
                        buf.push(0x01); // ATYP IPv4
                        buf.extend_from_slice(&ip.octets());
                    }
                    Address::Domain(dom) => {
                        buf.push(0x03); // ATYP Domain
                        buf.push(dom.len() as u8);
                        buf.extend_from_slice(dom.as_bytes());
                    }
                    Address::Ipv6(ip) => {
                        buf.push(0x04); // ATYP IPv6
                        buf.extend_from_slice(&ip.octets());
                    }
                }
                buf.extend_from_slice(&self.destination.port.to_be_bytes());
                buf.extend_from_slice(&(self.payload.len() as u16).to_be_bytes());
                buf.extend_from_slice(&self.payload);
            }
            UotVersion::Legacy => {
                match &self.destination.address {
                    Address::Ipv4(ip) => {
                        buf.push(0x01);
                        buf.extend_from_slice(&ip.octets());
                    }
                    Address::Domain(dom) => {
                        buf.push(0x03);
                        buf.push(dom.len() as u8);
                        buf.extend_from_slice(dom.as_bytes());
                    }
                    Address::Ipv6(ip) => {
                        buf.push(0x04);
                        buf.extend_from_slice(&ip.octets());
                    }
                }
                buf.extend_from_slice(&self.destination.port.to_be_bytes());
                buf.extend_from_slice(&(self.payload.len() as u16).to_be_bytes());
                buf.extend_from_slice(&self.payload);
            }
        }
        buf
    }

    pub fn decode(data: &[u8], version: UotVersion) -> Result<(Self, usize)> {
        if data.len() < 7 {
            return Err(Error::Protocol("UoT packet too short".into()));
        }

        let mut pos = 0;
        if version == UotVersion::Standard {
            let _cmd = data[0];
            pos += 1;
        }

        let atyp = data[pos];
        pos += 1;

        let address = match atyp {
            0x01 => {
                if pos + 4 > data.len() {
                    return Err(Error::Protocol("Incomplete IPv4 in UoT".into()));
                }
                let ip =
                    std::net::Ipv4Addr::new(data[pos], data[pos + 1], data[pos + 2], data[pos + 3]);
                pos += 4;
                Address::Ipv4(ip)
            }
            0x03 => {
                if pos + 1 > data.len() {
                    return Err(Error::Protocol("Incomplete Domain len in UoT".into()));
                }
                let len = data[pos] as usize;
                pos += 1;
                if pos + len > data.len() {
                    return Err(Error::Protocol("Incomplete Domain in UoT".into()));
                }
                let dom = std::str::from_utf8(&data[pos..pos + len])
                    .map_err(|_| Error::Protocol("Invalid UTF-8 domain in UoT".into()))?;
                pos += len;
                Address::Domain(dom.to_string())
            }
            0x04 => {
                if pos + 16 > data.len() {
                    return Err(Error::Protocol("Incomplete IPv6 in UoT".into()));
                }
                let mut octets = [0u8; 16];
                octets.copy_from_slice(&data[pos..pos + 16]);
                pos += 16;
                Address::Ipv6(std::net::Ipv6Addr::from(octets))
            }
            _ => {
                return Err(Error::Protocol(format!(
                    "Invalid ATYP 0x{:02x} in UoT",
                    atyp
                )));
            }
        };

        if pos + 4 > data.len() {
            return Err(Error::Protocol("Incomplete port/length in UoT".into()));
        }

        let port = u16::from_be_bytes([data[pos], data[pos + 1]]);
        let payload_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;

        if pos + payload_len > data.len() {
            return Err(Error::Protocol("Incomplete payload in UoT".into()));
        }

        let payload = data[pos..pos + payload_len].to_vec();
        pos += payload_len;

        Ok((
            UotPacket {
                destination: Destination::udp(address, port),
                payload,
            },
            pos,
        ))
    }
}

pub fn is_uot_destination(dest: &Destination) -> Option<UotVersion> {
    match &dest.address {
        Address::Domain(d) if d == UOT_MAGIC_ADDRESS => Some(UotVersion::Standard),
        Address::Domain(d) if d == UOT_LEGACY_MAGIC_ADDRESS => Some(UotVersion::Legacy),
        _ => None,
    }
}
