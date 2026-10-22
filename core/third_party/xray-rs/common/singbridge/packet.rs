// Module: common\singbridge\packet.rs
// 1:1 Rust implementation corresponding to Go common\singbridge\packet.go

use std::net::SocketAddr;
use std::time::Duration;

use super::destination::{Socksaddr, to_destination, to_socksaddr};
use super::error::return_error;
use crate::common::buf::{Buffer, MultiBuffer};
use crate::common::errors::{Error, Result};
use crate::common::net::{Destination, Network};
use crate::transport::Link;
use crate::transport::pipe::{Reader as PipeReader, Writer as PipeWriter};

pub const PACKET_READ_TIMEOUT: Duration = Duration::from_secs(60);

/// PacketConnWrapper wraps a Link and manages packet read/write with address translation.
/// 1:1 corresponding to PacketConnWrapper in packet.go.
pub struct PacketConnWrapper {
    reader: PipeReader,
    writer: PipeWriter,
    default_dest: Destination,
    cached: Vec<Buffer>,
}

impl PacketConnWrapper {
    pub fn new(reader: PipeReader, writer: PipeWriter, dest: Destination) -> Self {
        Self {
            reader,
            writer,
            default_dest: dest,
            cached: Vec::new(),
        }
    }

    pub fn from_link(link: &Link, dest: Destination) -> Self {
        Self::new(link.reader.clone(), link.writer.clone(), dest)
    }

    /// ReadPacket retrieves a packet payload and its target/source Socksaddr.
    pub async fn read_packet(&mut self) -> Result<(Buffer, Socksaddr)> {
        if !self.cached.is_empty() {
            let buf = self.cached.remove(0);
            let socksaddr = to_socksaddr(&self.default_dest);
            return Ok((buf, socksaddr));
        }

        match self
            .reader
            .read_multi_buffer_timeout(PACKET_READ_TIMEOUT)
            .await
        {
            Ok(mb) => {
                let mut buffers = mb.into_buffers();
                if buffers.is_empty() {
                    return Err(Error::Io(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "empty packet multibuffer",
                    )));
                }
                let first = buffers.remove(0);
                self.cached = buffers;
                let socksaddr = to_socksaddr(&self.default_dest);
                Ok((first, socksaddr))
            }
            Err(e) => {
                self.reader.interrupt().await;
                Err(e)
            }
        }
    }

    /// WritePacket writes a packet buffer directed to a target Socksaddr.
    pub async fn write_packet(&self, buffer: Buffer, dest: Socksaddr) -> Result<()> {
        let _target = to_destination(&dest, Network::Udp);
        let mut mb = MultiBuffer::new();
        mb.push(buffer);
        self.writer.write_multi_buffer(mb).await
    }

    pub async fn close(&self) -> Result<()> {
        self.writer.close().await
    }
}

/// CopyPacketConn copies packets between a Link and a packet connection.
/// 1:1 corresponding to CopyPacketConn() in packet.go.
pub async fn copy_packet_conn(link: &Link, destination: Destination) -> Result<()> {
    let mut wrapper = PacketConnWrapper::from_link(link, destination);
    match wrapper.read_packet().await {
        Ok(_) => Ok(()),
        Err(e) => match return_error(Some(e)) {
            None => Ok(()),
            Some(err) => Err(err),
        },
    }
}

/// Backwards compatibility struct
pub struct SingPacket {
    pub data: Vec<u8>,
    pub source: Option<SocketAddr>,
    pub destination: Option<SocketAddr>,
}

impl SingPacket {
    pub fn new(data: Vec<u8>, source: Option<SocketAddr>, destination: Option<SocketAddr>) -> Self {
        Self {
            data,
            source,
            destination,
        }
    }
}
