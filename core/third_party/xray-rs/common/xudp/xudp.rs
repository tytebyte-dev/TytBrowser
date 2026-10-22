// Module: common\xudp\xudp.rs
// 1:1 Rust implementation corresponding to Go common\xudp\xudp.go

use crate::common::errors::{Error, Result};
use crate::common::net::{Address, Destination};
use std::net::{Ipv4Addr, Ipv6Addr};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const XUDP_MAGIC: u16 = 0x5855;
pub const XUDP_MAX_PACKET_SIZE: usize = 65535;

pub fn write_address_port(buf: &mut Vec<u8>, addr: &Address, port: u16) {
    buf.extend_from_slice(&port.to_be_bytes());
    match addr {
        Address::Ipv4(v4) => {
            buf.push(1);
            buf.extend_from_slice(&v4.octets());
        }
        Address::Domain(dom) => {
            buf.push(2);
            buf.push(dom.len() as u8);
            buf.extend_from_slice(dom.as_bytes());
        }
        Address::Ipv6(v6) => {
            buf.push(3);
            buf.extend_from_slice(&v6.octets());
        }
    }
}

pub fn read_address_port(data: &[u8]) -> Result<(Address, u16, usize)> {
    if data.len() < 3 {
        return Err(Error::Protocol("short address buffer".into()));
    }
    let port = u16::from_be_bytes([data[0], data[1]]);
    let addr_type = data[2];
    match addr_type {
        1 => {
            if data.len() < 7 {
                return Err(Error::Protocol("short IPv4 buffer".into()));
            }
            let mut ip = [0u8; 4];
            ip.copy_from_slice(&data[3..7]);
            Ok((Address::Ipv4(Ipv4Addr::from(ip)), port, 7))
        }
        2 => {
            if data.len() < 4 {
                return Err(Error::Protocol("short domain length byte".into()));
            }
            let len = data[3] as usize;
            if data.len() < 4 + len {
                return Err(Error::Protocol("short domain buffer".into()));
            }
            let s = String::from_utf8(data[4..4 + len].to_vec())
                .map_err(|e| Error::Protocol(format!("invalid domain: {}", e)))?;
            Ok((Address::Domain(s), port, 4 + len))
        }
        3 => {
            if data.len() < 19 {
                return Err(Error::Protocol("short IPv6 buffer".into()));
            }
            let mut ip = [0u8; 16];
            ip.copy_from_slice(&data[3..19]);
            Ok((Address::Ipv6(Ipv6Addr::from(ip)), port, 19))
        }
        other => Err(Error::Protocol(format!(
            "unknown address family: {}",
            other
        ))),
    }
}

pub struct PacketWriter<W> {
    writer: W,
    dest: Destination,
    global_id: [u8; 8],
    is_first: bool,
}

impl<W: AsyncWrite + Unpin> PacketWriter<W> {
    pub fn new(writer: W, dest: Destination, global_id: [u8; 8]) -> Self {
        Self {
            writer,
            dest,
            global_id,
            is_first: true,
        }
    }

    pub fn destination(&self) -> &Destination {
        &self.dest
    }

    pub async fn write_packet(&mut self, payload: &[u8], is_proxy_req: bool) -> Result<()> {
        let mut meta = Vec::new();
        // 2 bytes placeholder for meta length
        meta.extend_from_slice(&[0, 0]);
        // 2 bytes Mux Session ID
        meta.extend_from_slice(&[0, 0]);

        if self.is_first {
            meta.push(1); // New
            meta.push(1); // Opt
            meta.push(2); // UDP
            write_address_port(&mut meta, &self.dest.address, self.dest.port);
            if is_proxy_req {
                meta.extend_from_slice(&self.global_id);
            }
            self.is_first = false;
        } else {
            meta.push(2); // Keep
            meta.push(1); // Opt
            meta.push(2); // UDP
            write_address_port(&mut meta, &self.dest.address, self.dest.port);
        }

        let meta_len = (meta.len() - 2) as u16;
        meta[0..2].copy_from_slice(&meta_len.to_be_bytes());

        let payload_len = payload.len() as u16;
        meta.extend_from_slice(&payload_len.to_be_bytes());
        meta.extend_from_slice(payload);

        self.writer.write_all(&meta).await.map_err(Error::Io)?;
        self.writer.flush().await.map_err(Error::Io)?;
        Ok(())
    }
}

pub struct PacketReader<R> {
    reader: R,
}

impl<R: AsyncRead + Unpin> PacketReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub async fn read_packet(&mut self) -> Result<(Destination, Vec<u8>)> {
        let mut len_buf = [0u8; 2];
        self.reader
            .read_exact(&mut len_buf)
            .await
            .map_err(Error::Io)?;
        let meta_len = u16::from_be_bytes(len_buf) as usize;
        if meta_len < 4 {
            return Err(Error::Protocol("XUDP meta too short".into()));
        }

        let mut meta = vec![0u8; meta_len];
        self.reader.read_exact(&mut meta).await.map_err(Error::Io)?;

        let mut dest = Destination::udp(Address::Ipv4(Ipv4Addr::UNSPECIFIED), 0);
        if meta.len() > 4 && meta[4] == 2 {
            let (addr, port, _) = read_address_port(&meta[5..])?;
            dest = Destination::udp(addr, port);
        }

        // Read payload length
        self.reader
            .read_exact(&mut len_buf)
            .await
            .map_err(Error::Io)?;
        let payload_len = u16::from_be_bytes(len_buf) as usize;
        let mut payload = vec![0u8; payload_len];
        self.reader
            .read_exact(&mut payload)
            .await
            .map_err(Error::Io)?;

        Ok((dest, payload))
    }
}
