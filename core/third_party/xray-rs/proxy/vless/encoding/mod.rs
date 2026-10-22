pub mod addons;
#[path = "addons.pb.rs"]
pub mod addons_pb;
pub mod encoding;

#[cfg(test)]
pub mod encoding_test;

use crate::common::errors::{Error, Result};
use crate::common::net::{Address, Destination};
use crate::common::protocol::RequestCommand;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use uuid::Uuid;

pub const VLESS_VERSION: u8 = 0;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Addons {
    pub flow: String,
    pub seed: Vec<u8>,
}

impl Addons {
    pub fn new(flow: impl Into<String>) -> Self {
        Self {
            flow: flow.into(),
            seed: Vec::new(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        if !self.flow.is_empty() {
            let flow_bytes = self.flow.as_bytes();
            buf.push((1 << 3) | 2);
            buf.push(flow_bytes.len() as u8);
            buf.extend_from_slice(flow_bytes);
        }
        if !self.seed.is_empty() {
            buf.push((2 << 3) | 2);
            buf.push(self.seed.len() as u8);
            buf.extend_from_slice(&self.seed);
        }
        buf
    }

    pub fn decode(bytes: &[u8]) -> Self {
        let mut flow = String::new();
        let mut seed = Vec::new();
        let mut idx = 0;

        while idx < bytes.len() {
            let tag_wire = bytes[idx];
            idx += 1;
            let field_num = tag_wire >> 3;
            if idx >= bytes.len() {
                break;
            }
            let len = bytes[idx] as usize;
            idx += 1;
            if idx + len > bytes.len() {
                break;
            }
            let data = &bytes[idx..idx + len];
            idx += len;

            match field_num {
                1 => flow = String::from_utf8_lossy(data).to_string(),
                2 => seed = data.to_vec(),
                _ => {}
            }
        }

        Self { flow, seed }
    }
}

#[derive(Debug, Clone)]
pub struct RequestHeader {
    pub user_id: Uuid,
    pub command: RequestCommand,
    pub destination: Destination,
    pub addons: Addons,
}

impl RequestHeader {
    pub fn new(user_id: Uuid, command: RequestCommand, destination: Destination) -> Self {
        Self {
            user_id,
            command,
            destination,
            addons: Addons::default(),
        }
    }

    pub async fn encode<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<()> {
        let mut buf = Vec::new();
        buf.push(VLESS_VERSION);
        buf.extend_from_slice(self.user_id.as_bytes());

        let addons_bytes = self.addons.encode();
        buf.push(addons_bytes.len() as u8);
        buf.extend_from_slice(&addons_bytes);

        buf.push(self.command.to_u8());
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

        writer.write_all(&buf).await?;
        writer.flush().await?;
        Ok(())
    }

    pub async fn decode<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
        let version = reader.read_u8().await?;
        if version != VLESS_VERSION {
            return Err(Error::Protocol(format!(
                "Unsupported VLESS version: {}",
                version
            )));
        }

        let mut user_bytes = [0u8; 16];
        reader.read_exact(&mut user_bytes).await?;
        let user_id = Uuid::from_bytes(user_bytes);

        let addons_len = reader.read_u8().await? as usize;
        let addons = if addons_len > 0 {
            let mut addons_buf = vec![0u8; addons_len];
            reader.read_exact(&mut addons_buf).await?;
            Addons::decode(&addons_buf)
        } else {
            Addons::default()
        };

        let cmd_byte = reader.read_u8().await?;
        let command = RequestCommand::from_u8(cmd_byte)
            .ok_or_else(|| Error::Protocol(format!("Invalid VLESS command: {}", cmd_byte)))?;

        let port = reader.read_u16().await?;
        let addr_type = reader.read_u8().await?;
        let address = match addr_type {
            1 => {
                let mut ip = [0u8; 4];
                reader.read_exact(&mut ip).await?;
                Address::Ipv4(std::net::Ipv4Addr::from(ip))
            }
            2 => {
                let len = reader.read_u8().await? as usize;
                let mut dom = vec![0u8; len];
                reader.read_exact(&mut dom).await?;
                let s = String::from_utf8(dom).map_err(|e| {
                    Error::Protocol(format!("Invalid UTF-8 domain in VLESS header: {}", e))
                })?;
                Address::Domain(s)
            }
            3 => {
                let mut ip = [0u8; 16];
                reader.read_exact(&mut ip).await?;
                Address::Ipv6(std::net::Ipv6Addr::from(ip))
            }
            other => {
                return Err(Error::Protocol(format!(
                    "Invalid VLESS address type: {}",
                    other
                )));
            }
        };

        let destination = if command == RequestCommand::Udp {
            Destination::udp(address, port)
        } else {
            Destination::tcp(address, port)
        };

        Ok(Self {
            user_id,
            command,
            destination,
            addons,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResponseHeader {
    pub addons: Addons,
}

impl ResponseHeader {
    pub fn new() -> Self {
        Self {
            addons: Addons::default(),
        }
    }

    pub async fn encode<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<()> {
        let mut buf = Vec::new();
        buf.push(VLESS_VERSION);
        let addons_bytes = self.addons.encode();
        buf.push(addons_bytes.len() as u8);
        buf.extend_from_slice(&addons_bytes);

        writer.write_all(&buf).await?;
        writer.flush().await?;
        Ok(())
    }

    pub async fn decode<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
        let version = reader.read_u8().await?;
        if version != VLESS_VERSION {
            return Err(Error::Protocol(format!(
                "Unsupported VLESS response version: {}",
                version
            )));
        }
        let addons_len = reader.read_u8().await? as usize;
        let addons = if addons_len > 0 {
            let mut buf = vec![0u8; addons_len];
            reader.read_exact(&mut buf).await?;
            Addons::decode(&buf)
        } else {
            Addons::default()
        };
        Ok(Self { addons })
    }
}
