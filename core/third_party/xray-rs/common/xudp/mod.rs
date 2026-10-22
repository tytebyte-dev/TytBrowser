pub mod xudp;

#[cfg(test)]
pub mod xudp_test;

pub use xudp::{
    PacketReader, PacketWriter, XUDP_MAGIC, XUDP_MAX_PACKET_SIZE, read_address_port,
    write_address_port,
};

use crate::common::errors::{Error, Result};
use crate::common::net::{Address, Destination};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub fn generate_global_id(src: &SocketAddr, base_key: &[u8; 32]) -> [u8; 8] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(base_key);
    hasher.update(src.to_string().as_bytes());
    let res = hasher.finalize();
    let mut out = [0u8; 8];
    out.copy_from_slice(&res[0..8]);
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XudpPacket {
    pub destination: Destination,
    pub payload: Vec<u8>,
}

impl XudpPacket {
    pub fn new(destination: Destination, payload: Vec<u8>) -> Self {
        Self {
            destination,
            payload,
        }
    }

    pub async fn encode<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<()> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.destination.port.to_be_bytes());

        match &self.destination.address {
            Address::Ipv4(v4) => {
                buf.push(1); // IPv4
                buf.extend_from_slice(&v4.octets());
            }
            Address::Domain(dom) => {
                buf.push(2); // Domain
                buf.push(dom.len() as u8);
                buf.extend_from_slice(dom.as_bytes());
            }
            Address::Ipv6(v6) => {
                buf.push(3); // IPv6
                buf.extend_from_slice(&v6.octets());
            }
        }

        buf.extend_from_slice(&self.payload);
        writer.write_all(&buf).await?;
        writer.flush().await?;
        Ok(())
    }

    pub async fn decode<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
        let port = reader.read_u16().await?;
        let addr_type = reader.read_u8().await?;
        let address = match addr_type {
            1 => {
                let mut ip = [0u8; 4];
                reader.read_exact(&mut ip).await?;
                Address::Ipv4(Ipv4Addr::from(ip))
            }
            2 => {
                let len = reader.read_u8().await? as usize;
                let mut dom = vec![0u8; len];
                reader.read_exact(&mut dom).await?;
                let s = String::from_utf8(dom)
                    .map_err(|e| Error::Protocol(format!("Invalid UTF-8 domain in XUDP: {}", e)))?;
                Address::Domain(s)
            }
            3 => {
                let mut ip = [0u8; 16];
                reader.read_exact(&mut ip).await?;
                Address::Ipv6(Ipv6Addr::from(ip))
            }
            other => {
                return Err(Error::Protocol(format!(
                    "Invalid XUDP address type: {}",
                    other
                )));
            }
        };

        let mut payload = Vec::new();
        reader.read_to_end(&mut payload).await?;

        Ok(Self {
            destination: Destination::udp(address, port),
            payload,
        })
    }
}
