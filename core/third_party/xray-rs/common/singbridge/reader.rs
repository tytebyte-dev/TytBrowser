// Module: common\singbridge\reader.rs
// 1:1 Rust implementation corresponding to Go common\singbridge\reader.go

use async_trait::async_trait;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;

use crate::common::buf::{Buffer, MultiBuffer, Reader, TimeoutReader, Writer};
use crate::common::errors::{Error, Result};
use crate::common::net::BoxStream;

/// Conn wraps an async stream and adapts it to Xray MultiBuffer Reader/Writer traits.
/// 1:1 corresponding to Conn in reader.go.
pub struct Conn {
    stream: BoxStream,
}

impl Conn {
    pub fn new(stream: BoxStream) -> Self {
        Self { stream }
    }

    pub fn into_inner(self) -> BoxStream {
        self.stream
    }
}

#[async_trait]
impl Reader for Conn {
    async fn read_multi_buffer(&mut self) -> Result<MultiBuffer> {
        let mut raw = [0u8; 8192];
        let n = self.stream.read(&mut raw).await.map_err(Error::Io)?;
        if n == 0 {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "eof",
            )));
        }
        let buf = Buffer::from_bytes(&raw[..n]);
        let mut mb = MultiBuffer::new();
        mb.push(buf);
        Ok(mb)
    }
}

#[async_trait]
impl TimeoutReader for Conn {
    async fn read_multi_buffer_timeout(&mut self, duration: Duration) -> Result<MultiBuffer> {
        match timeout(duration, self.read_multi_buffer()).await {
            Ok(res) => res,
            Err(_) => Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "read timeout",
            ))),
        }
    }
}

#[async_trait]
impl Writer for Conn {
    async fn write_multi_buffer(&mut self, mb: MultiBuffer) -> Result<()> {
        for buffer in mb.buffers() {
            let slice = buffer.as_slice();
            if !slice.is_empty() {
                self.stream.write_all(slice).await.map_err(Error::Io)?;
            }
        }
        self.stream.flush().await.map_err(Error::Io)?;
        Ok(())
    }
}

/// Backwards-compatibility wrapper
pub struct SingReader<R> {
    inner: R,
}

impl<R: AsyncRead + Unpin> SingReader<R> {
    pub fn new(inner: R) -> Self {
        Self { inner }
    }

    pub async fn read_bytes(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf).await.map_err(Error::Io)
    }
}
