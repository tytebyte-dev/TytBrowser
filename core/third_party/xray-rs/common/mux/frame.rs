// Module: common\mux\frame.rs
// 1:1 Rust implementation corresponding to Go common\mux\frame.go

use crate::common::errors::{Error, Result};
use crate::common::net::{Address, Destination, Network};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SessionStatus {
    New = 0x01,
    Keep = 0x02,
    End = 0x03,
    KeepAlive = 0x04,
}

impl SessionStatus {
    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(SessionStatus::New),
            0x02 => Some(SessionStatus::Keep),
            0x03 => Some(SessionStatus::End),
            0x04 => Some(SessionStatus::KeepAlive),
            _ => None,
        }
    }
}

pub const OPTION_DATA: u8 = 0x01;
pub const OPTION_ERROR: u8 = 0x02;

pub const TARGET_NETWORK_TCP: u8 = 0x01;
pub const TARGET_NETWORK_UDP: u8 = 0x02;

/// FrameMetadata corresponds 1:1 to Go `mux.FrameMetadata`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameMetadata {
    pub target: Option<Destination>,
    pub session_id: u16,
    pub option: u8,
    pub session_status: SessionStatus,
    pub global_id: [u8; 8],
}

impl FrameMetadata {
    pub fn new(session_id: u16, status: SessionStatus) -> Self {
        Self {
            target: None,
            session_id,
            option: 0,
            session_status: status,
            global_id: [0u8; 8],
        }
    }

    pub fn write_to(&self, buf: &mut Vec<u8>) -> Result<()> {
        let len_pos = buf.len();
        buf.extend_from_slice(&[0u8, 0u8]); // Placeholder for 2-byte length

        let meta_start = buf.len();
        buf.extend_from_slice(&self.session_id.to_be_bytes());
        buf.push(self.session_status as u8);
        buf.push(self.option);

        if self.session_status == SessionStatus::New
            && let Some(target) = &self.target
        {
            let net_byte = match target.network {
                Network::Tcp => TARGET_NETWORK_TCP,
                Network::Udp => TARGET_NETWORK_UDP,
            };
            buf.push(net_byte);
            buf.extend_from_slice(&target.port.to_be_bytes());
            match &target.address {
                Address::Ipv4(v4) => {
                    buf.push(1); // IPv4
                    buf.extend_from_slice(&v4.octets());
                }
                Address::Ipv6(v6) => {
                    buf.push(2); // IPv6
                    buf.extend_from_slice(&v6.octets());
                }
                Address::Domain(domain) => {
                    buf.push(3); // Domain
                    buf.push(domain.len() as u8);
                    buf.extend_from_slice(domain.as_bytes());
                }
            }
        }

        let meta_len = (buf.len() - meta_start) as u16;
        buf[len_pos..len_pos + 2].copy_from_slice(&meta_len.to_be_bytes());
        Ok(())
    }

    pub fn unmarshal(&mut self, bytes: &[u8]) -> Result<usize> {
        if bytes.len() < 2 {
            return Err(Error::Protocol(
                "Insufficient buffer for meta length".into(),
            ));
        }
        let meta_len = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;
        if bytes.len() < 2 + meta_len {
            return Err(Error::Protocol("Insufficient buffer for metadata".into()));
        }
        if meta_len < 4 {
            return Err(Error::Protocol("Meta length too short".into()));
        }

        let slice = &bytes[2..2 + meta_len];
        self.session_id = u16::from_be_bytes([slice[0], slice[1]]);
        self.session_status = SessionStatus::from_u8(slice[2])
            .ok_or_else(|| Error::Protocol(format!("Invalid session status: {}", slice[2])))?;
        self.option = slice[3];

        if self.session_status == SessionStatus::New && slice.len() >= 8 {
            let net_byte = slice[4];
            let network = if net_byte == TARGET_NETWORK_UDP {
                Network::Udp
            } else {
                Network::Tcp
            };
            let port = u16::from_be_bytes([slice[5], slice[6]]);
            let addr_type = slice[7];
            let mut offset = 8;

            let address = match addr_type {
                1 => {
                    if slice.len() < offset + 4 {
                        return Err(Error::Protocol("Short IPv4 address".into()));
                    }
                    let mut octets = [0u8; 4];
                    octets.copy_from_slice(&slice[offset..offset + 4]);
                    Address::Ipv4(std::net::Ipv4Addr::from(octets))
                }
                2 => {
                    if slice.len() < offset + 16 {
                        return Err(Error::Protocol("Short IPv6 address".into()));
                    }
                    let mut octets = [0u8; 16];
                    octets.copy_from_slice(&slice[offset..offset + 16]);
                    Address::Ipv6(std::net::Ipv6Addr::from(octets))
                }
                3 => {
                    if slice.len() < offset + 1 {
                        return Err(Error::Protocol("Short domain length".into()));
                    }
                    let dlen = slice[offset] as usize;
                    offset += 1;
                    if slice.len() < offset + dlen {
                        return Err(Error::Protocol("Short domain string".into()));
                    }
                    let s = String::from_utf8(slice[offset..offset + dlen].to_vec())
                        .map_err(|e| Error::Protocol(format!("Invalid domain: {}", e)))?;
                    Address::Domain(s)
                }
                other => return Err(Error::Protocol(format!("Unknown address type: {}", other))),
            };

            self.target = Some(if network == Network::Udp {
                Destination::udp(address, port)
            } else {
                Destination::tcp(address, port)
            });
        }

        Ok(2 + meta_len)
    }
}

/// Frame represents a complete Mux protocol frame with metadata and optional payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub session_id: u16,
    pub status: SessionStatus,
    pub option: u8,
    pub target: Option<Destination>,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn new(session_id: u16, status: SessionStatus, payload: Vec<u8>) -> Self {
        Self {
            session_id,
            status,
            option: if !payload.is_empty() { OPTION_DATA } else { 0 },
            target: None,
            payload,
        }
    }

    pub fn new_session(session_id: u16, target: Destination, payload: Vec<u8>) -> Self {
        Self {
            session_id,
            status: SessionStatus::New,
            option: OPTION_DATA,
            target: Some(target),
            payload,
        }
    }

    pub fn data(session_id: u16, payload: Vec<u8>) -> Self {
        Self {
            session_id,
            status: SessionStatus::Keep,
            option: OPTION_DATA,
            target: None,
            payload,
        }
    }

    pub fn end(session_id: u16) -> Self {
        Self {
            session_id,
            status: SessionStatus::End,
            option: 0,
            target: None,
            payload: Vec::new(),
        }
    }

    pub async fn encode<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<()> {
        let mut header_buf = Vec::new();
        header_buf.extend_from_slice(&self.session_id.to_be_bytes());
        header_buf.push(self.status as u8);
        header_buf.push(self.option);

        if self.status == SessionStatus::New
            && let Some(target) = &self.target
        {
            let net_byte = match target.network {
                Network::Tcp => TARGET_NETWORK_TCP,
                Network::Udp => TARGET_NETWORK_UDP,
            };
            header_buf.push(net_byte);
            header_buf.extend_from_slice(&target.port.to_be_bytes());
            match &target.address {
                Address::Ipv4(v4) => {
                    header_buf.push(1);
                    header_buf.extend_from_slice(&v4.octets());
                }
                Address::Ipv6(v6) => {
                    header_buf.push(2);
                    header_buf.extend_from_slice(&v6.octets());
                }
                Address::Domain(domain) => {
                    header_buf.push(3);
                    header_buf.push(domain.len() as u8);
                    header_buf.extend_from_slice(domain.as_bytes());
                }
            }
        }

        let total_frame_len = header_buf.len() + self.payload.len();
        if total_frame_len > u16::MAX as usize {
            return Err(Error::BufferOverflow);
        }

        writer.write_u16(total_frame_len as u16).await?;
        writer.write_all(&header_buf).await?;
        if !self.payload.is_empty() {
            writer.write_all(&self.payload).await?;
        }
        writer.flush().await?;
        Ok(())
    }

    pub async fn write_to<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<()> {
        self.encode(writer).await
    }

    pub async fn read_from<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
        Self::decode(reader).await
    }

    pub async fn decode<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
        let frame_len = reader.read_u16().await? as usize;
        if frame_len < 4 {
            return Err(Error::Protocol("Mux frame length too short".into()));
        }

        let session_id = reader.read_u16().await?;
        let status_byte = reader.read_u8().await?;
        let status = SessionStatus::from_u8(status_byte)
            .ok_or_else(|| Error::Protocol(format!("Invalid Mux status: {}", status_byte)))?;
        let option = reader.read_u8().await?;

        let mut bytes_read = 4;
        let mut target = None;

        if status == SessionStatus::New {
            let net_byte = reader.read_u8().await?;
            let network = if net_byte == TARGET_NETWORK_UDP {
                Network::Udp
            } else {
                Network::Tcp
            };
            let port = reader.read_u16().await?;
            let addr_type = reader.read_u8().await?;
            bytes_read += 4;

            let address = match addr_type {
                1 => {
                    let mut ip = [0u8; 4];
                    reader.read_exact(&mut ip).await?;
                    bytes_read += 4;
                    Address::Ipv4(std::net::Ipv4Addr::from(ip))
                }
                2 => {
                    let mut ip = [0u8; 16];
                    reader.read_exact(&mut ip).await?;
                    bytes_read += 16;
                    Address::Ipv6(std::net::Ipv6Addr::from(ip))
                }
                3 => {
                    let len = reader.read_u8().await? as usize;
                    let mut dom = vec![0u8; len];
                    reader.read_exact(&mut dom).await?;
                    bytes_read += 1 + len;
                    let s = String::from_utf8(dom)
                        .map_err(|e| Error::Protocol(format!("Invalid UTF-8 domain: {}", e)))?;
                    Address::Domain(s)
                }
                other => {
                    return Err(Error::Protocol(format!(
                        "Invalid Mux address type: {}",
                        other
                    )));
                }
            };
            target = Some(if network == Network::Udp {
                Destination::udp(address, port)
            } else {
                Destination::tcp(address, port)
            });
        }

        let payload_len = frame_len.saturating_sub(bytes_read);
        let mut payload = vec![0u8; payload_len];
        if payload_len > 0 {
            reader.read_exact(&mut payload).await?;
        }

        Ok(Self {
            session_id,
            status,
            option,
            target,
            payload,
        })
    }
}

pub type FrameType = SessionStatus;
