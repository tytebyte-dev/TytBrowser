// Module: common\mux\reader.rs
// 1:1 Rust implementation corresponding to Go common\mux\reader.go

use super::frame::Frame;
use crate::common::errors::Result;
use crate::common::net::Destination;
use tokio::io::{AsyncRead, AsyncReadExt};

/// PacketReader reads a chunk of Mux frames.
pub struct PacketReader<R> {
    reader: R,
    eof: bool,
    dest: Option<Destination>,
}

impl<R: AsyncRead + Unpin> PacketReader<R> {
    pub fn new(reader: R, dest: Option<Destination>) -> Self {
        Self {
            reader,
            eof: false,
            dest,
        }
    }

    pub fn destination(&self) -> Option<&Destination> {
        self.dest.as_ref()
    }

    pub async fn read_packet(&mut self) -> Result<Option<Vec<u8>>> {
        if self.eof {
            return Ok(None);
        }
        let size = match self.reader.read_u16().await {
            Ok(s) => s as usize,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                self.eof = true;
                return Ok(None);
            }
            Err(e) => return Err(e.into()),
        };

        let mut buf = vec![0u8; size];
        self.reader.read_exact(&mut buf).await?;
        self.eof = true;
        Ok(Some(buf))
    }
}

/// StreamReader reads chunks using plain 2-byte chunk size framing.
pub struct StreamReader<R> {
    reader: R,
}

impl<R: AsyncRead + Unpin> StreamReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub async fn read_chunk(&mut self) -> Result<Option<Vec<u8>>> {
        let size = match self.reader.read_u16().await {
            Ok(s) => s as usize,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        if size == 0 {
            return Ok(None);
        }
        let mut buf = vec![0u8; size];
        self.reader.read_exact(&mut buf).await?;
        Ok(Some(buf))
    }
}

/// FrameReader reads Frames from an underlying async reader.
pub struct FrameReader<R> {
    reader: R,
}

impl<R: AsyncRead + Unpin> FrameReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub async fn read_frame(&mut self) -> Result<Frame> {
        Frame::read_from(&mut self.reader).await
    }
}
