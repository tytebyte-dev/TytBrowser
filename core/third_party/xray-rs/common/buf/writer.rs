// Module: common\buf\writer.rs
// 1:1 Rust implementation corresponding to Go common\buf\writer.go

use async_trait::async_trait;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::common::buf::io::Writer;
use crate::common::buf::{Buffer, MultiBuffer};
use crate::common::errors::{Error, Result};

pub struct BufferedWriter<W> {
    inner: W,
    buffer: Buffer,
    buffered: bool,
}

impl<W: AsyncWrite + Unpin> BufferedWriter<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            buffer: Buffer::new(),
            buffered: true,
        }
    }

    pub fn into_inner(self) -> W {
        self.inner
    }

    pub fn set_buffered(&mut self, buffered: bool) {
        self.buffered = buffered;
    }

    pub async fn write_buffer(&mut self, buffer: &Buffer) -> Result<()> {
        if !self.buffered {
            self.inner
                .write_all(buffer.as_slice())
                .await
                .map_err(Error::Io)?;
            return Ok(());
        }

        if self.buffer.remaining_capacity() < buffer.len() {
            self.flush().await?;
        }
        let _ = self.buffer.write(buffer.as_slice());
        Ok(())
    }

    pub async fn write_bytes(&mut self, data: &[u8]) -> Result<()> {
        if !self.buffered {
            self.inner.write_all(data).await.map_err(Error::Io)?;
            return Ok(());
        }

        let mut remaining = data;
        while !remaining.is_empty() {
            if self.buffer.remaining_capacity() == 0 {
                self.flush().await?;
            }
            let written = self.buffer.write(remaining)?;
            remaining = &remaining[written..];
        }
        Ok(())
    }

    pub async fn write_multi_buffer(&mut self, mb: &MultiBuffer) -> Result<()> {
        for buf in mb.buffers() {
            self.write_buffer(buf).await?;
        }
        Ok(())
    }

    pub async fn flush(&mut self) -> Result<()> {
        if !self.buffer.is_empty() {
            self.inner
                .write_all(self.buffer.as_slice())
                .await
                .map_err(Error::Io)?;
            self.buffer.clear();
        }
        self.inner.flush().await.map_err(Error::Io)
    }
}

#[async_trait]
impl<W: AsyncWrite + Send + Sync + Unpin> Writer for BufferedWriter<W> {
    async fn write_multi_buffer(&mut self, mb: MultiBuffer) -> Result<()> {
        for buf in mb.buffers() {
            self.write_buffer(buf).await?;
        }
        Ok(())
    }
}

pub struct SequentialWriter<W> {
    inner: W,
}

impl<W: AsyncWrite + Unpin> SequentialWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<W: AsyncWrite + Send + Sync + Unpin> Writer for SequentialWriter<W> {
    async fn write_multi_buffer(&mut self, mb: MultiBuffer) -> Result<()> {
        for buf in mb.buffers() {
            self.inner
                .write_all(buf.as_slice())
                .await
                .map_err(Error::Io)?;
        }
        self.inner.flush().await.map_err(Error::Io)?;
        Ok(())
    }
}

pub struct Discard;

#[async_trait]
impl Writer for Discard {
    async fn write_multi_buffer(&mut self, _mb: MultiBuffer) -> Result<()> {
        Ok(())
    }
}
