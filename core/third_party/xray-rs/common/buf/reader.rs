// Module: common\buf\reader.rs
// 1:1 Rust implementation corresponding to Go common\buf\reader.go

use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::common::buf::buffer::Buffer;
use crate::common::buf::io::Reader;
use crate::common::buf::multi_buffer::MultiBuffer;
use crate::common::errors::{Error, Result};

pub async fn read_buffer<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Buffer> {
    let mut raw = [0u8; 8192];
    let n = reader.read(&mut raw).await.map_err(Error::Io)?;
    if n == 0 {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "eof",
        )));
    }
    let mut b = Buffer::with_capacity(n);
    let _ = b.write(&raw[..n]);
    Ok(b)
}

pub struct BufferedReader<R> {
    inner: R,
    cached: MultiBuffer,
    buffer_size: usize,
}

impl<R: AsyncRead + Unpin> BufferedReader<R> {
    pub fn new(inner: R, buffer_size: usize) -> Self {
        Self {
            inner,
            cached: MultiBuffer::new(),
            buffer_size: buffer_size.max(2048),
        }
    }

    pub fn buffered_bytes(&self) -> usize {
        self.cached.len()
    }

    pub async fn read_buffer(&mut self) -> Result<Option<Buffer>> {
        let mut buf = vec![0u8; self.buffer_size];
        let n = self.inner.read(&mut buf).await.map_err(Error::Io)?;
        if n == 0 {
            Ok(None)
        } else {
            Ok(Some(Buffer::from_bytes(&buf[..n])))
        }
    }

    pub async fn read_at_most(&mut self, max_size: usize) -> Result<MultiBuffer> {
        if self.cached.is_empty()
            && let Some(b) = self.read_buffer().await?
        {
            self.cached.push(b);
        }

        let mut out = MultiBuffer::new();
        let mut needed = max_size;

        let buffers = self.cached.clone().into_buffers();
        self.cached.clear();

        for mut b in buffers {
            if needed == 0 {
                self.cached.push(b);
                continue;
            }

            if b.len() <= needed {
                needed -= b.len();
                out.push(b);
            } else {
                let mut chunk = Buffer::with_capacity(needed);
                let mut data = vec![0u8; needed];
                let _ = b.read(&mut data);
                let _ = chunk.write(&data);
                out.push(chunk);
                self.cached.push(b);
                needed = 0;
            }
        }

        Ok(out)
    }
}

#[async_trait]
impl<R: AsyncRead + Send + Sync + Unpin> Reader for BufferedReader<R> {
    async fn read_multi_buffer(&mut self) -> Result<MultiBuffer> {
        if !self.cached.is_empty() {
            let mb = self.cached.clone();
            self.cached.clear();
            return Ok(mb);
        }

        match self.read_buffer().await? {
            Some(b) => {
                let mut mb = MultiBuffer::new();
                mb.push(b);
                Ok(mb)
            }
            None => Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "eof",
            ))),
        }
    }
}

pub struct SingleReader<R> {
    inner: R,
}

impl<R: AsyncRead + Unpin> SingleReader<R> {
    pub fn new(inner: R) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<R: AsyncRead + Send + Sync + Unpin> Reader for SingleReader<R> {
    async fn read_multi_buffer(&mut self) -> Result<MultiBuffer> {
        let b = read_buffer(&mut self.inner).await?;
        let mut mb = MultiBuffer::new();
        mb.push(b);
        Ok(mb)
    }
}

pub struct PacketReader<R> {
    inner: R,
}

impl<R: AsyncRead + Unpin> PacketReader<R> {
    pub fn new(inner: R) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<R: AsyncRead + Send + Sync + Unpin> Reader for PacketReader<R> {
    async fn read_multi_buffer(&mut self) -> Result<MultiBuffer> {
        let b = read_buffer(&mut self.inner).await?;
        let mut mb = MultiBuffer::new();
        mb.push(b);
        Ok(mb)
    }
}
