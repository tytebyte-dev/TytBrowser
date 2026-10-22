pub mod auth;
pub mod commands;
pub mod encoding;

#[cfg(test)]
pub mod encoding_test;

use crate::common::errors::{Error, Result};
use crate::common::net::{Address, Destination};
use crate::common::protocol::RequestCommand;
use md5::{Digest, Md5};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use uuid::Uuid;

pub const VMESS_VERSION: u8 = 1;

pub fn fnv1a_32(bytes: &[u8]) -> u32 {
    let mut hash: u32 = 0x811c9dc5;
    for &b in bytes {
        hash ^= b as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}

pub fn generate_chacha20_key(seed16: &[u8; 16]) -> [u8; 32] {
    let mut key = [0u8; 32];
    let mut hasher = Md5::new();
    hasher.update(seed16);
    let hash1 = hasher.finalize();
    key[0..16].copy_from_slice(&hash1);

    let mut hasher2 = Md5::new();
    hasher2.update(hash1);
    let hash2 = hasher2.finalize();
    key[16..32].copy_from_slice(&hash2);
    key
}

#[derive(Debug, Clone)]
pub struct RequestHeader {
    pub version: u8,
    pub user_id: Uuid,
    pub request_body_iv: [u8; 16],
    pub request_body_key: [u8; 16],
    pub response_header_byte: u8,
    pub command: RequestCommand,
    pub destination: Destination,
}

impl RequestHeader {
    pub fn new(user_id: Uuid, command: RequestCommand, destination: Destination) -> Self {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut request_body_iv = [0u8; 16];
        let mut request_body_key = [0u8; 16];
        rng.fill_bytes(&mut request_body_iv);
        rng.fill_bytes(&mut request_body_key);

        Self {
            version: VMESS_VERSION,
            user_id,
            request_body_iv,
            request_body_key,
            response_header_byte: rand::random(),
            command,
            destination,
        }
    }

    pub async fn encode<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<()> {
        let mut buf = Vec::with_capacity(128);
        buf.extend_from_slice(self.user_id.as_bytes());
        buf.push(self.version);
        buf.extend_from_slice(&self.request_body_iv);
        buf.extend_from_slice(&self.request_body_key);
        buf.push(self.response_header_byte);
        buf.push(0x01); // Options: ChunkStream
        buf.push(0x03); // Security: ChaCha20-Poly1305
        buf.push(0x00); // Reserved

        let cmd_byte = match self.command {
            RequestCommand::Tcp => 1,
            RequestCommand::Udp => 2,
            RequestCommand::Mux | RequestCommand::Rvs => 3,
        };
        buf.push(cmd_byte);
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

        let checksum = fnv1a_32(&buf);
        buf.extend_from_slice(&checksum.to_be_bytes());

        writer.write_all(&buf).await?;
        writer.flush().await?;
        Ok(())
    }

    pub async fn decode<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
        let mut user_id_bytes = [0u8; 16];
        reader.read_exact(&mut user_id_bytes).await?;
        let user_id = Uuid::from_bytes(user_id_bytes);

        let version = reader.read_u8().await?;
        if version != VMESS_VERSION {
            return Err(Error::Protocol(format!(
                "Unsupported VMess version: {}",
                version
            )));
        }

        let mut request_body_iv = [0u8; 16];
        reader.read_exact(&mut request_body_iv).await?;

        let mut request_body_key = [0u8; 16];
        reader.read_exact(&mut request_body_key).await?;

        let response_header_byte = reader.read_u8().await?;
        let _options = reader.read_u8().await?;
        let _security = reader.read_u8().await?;
        let _reserved = reader.read_u8().await?;

        let cmd_byte = reader.read_u8().await?;
        let command = match cmd_byte {
            1 => RequestCommand::Tcp,
            2 => RequestCommand::Udp,
            3 => RequestCommand::Mux,
            other => return Err(Error::Protocol(format!("Invalid VMess command: {}", other))),
        };

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
                    Error::Protocol(format!("Invalid UTF-8 domain in VMess header: {}", e))
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
                    "Invalid VMess address type: {}",
                    other
                )));
            }
        };

        let _checksum = reader.read_u32().await?;

        let destination = if command == RequestCommand::Udp {
            Destination::udp(address, port)
        } else {
            Destination::tcp(address, port)
        };

        Ok(Self {
            version,
            user_id,
            request_body_iv,
            request_body_key,
            response_header_byte,
            command,
            destination,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResponseHeader {
    pub response_header_byte: u8,
    pub command: Option<RequestCommand>,
}

impl ResponseHeader {
    pub fn new(response_header_byte: u8) -> Self {
        Self {
            response_header_byte,
            command: None,
        }
    }

    pub async fn encode<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<()> {
        let mut buf = Vec::with_capacity(4);
        buf.push(self.response_header_byte);
        buf.push(0); // Option
        buf.push(0); // Command
        buf.push(0); // Command len
        writer.write_all(&buf).await?;
        writer.flush().await?;
        Ok(())
    }

    pub async fn decode<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
        let response_header_byte = reader.read_u8().await?;
        let _opt = reader.read_u8().await?;
        let _cmd = reader.read_u8().await?;
        let cmd_len = reader.read_u8().await? as usize;
        if cmd_len > 0 {
            let mut discard = vec![0u8; cmd_len];
            reader.read_exact(&mut discard).await?;
        }

        Ok(Self {
            response_header_byte,
            command: None,
        })
    }
}
