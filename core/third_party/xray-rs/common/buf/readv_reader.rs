// Module: common\buf\readv_reader.rs
// 1:1 Rust implementation corresponding to Go common\buf\readv_reader.go

use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncReadExt};

use super::buffer::Buffer;
use super::io::Reader;
use super::multi_buffer::MultiBuffer;
use crate::common::errors::{Error, Result};

#[derive(Debug, Clone)]
pub struct AllocStrategy {
    current: u32,
}

impl AllocStrategy {
    pub fn new() -> Self {
        Self { current: 1 }
    }

    pub fn current(&self) -> u32 {
        self.current
    }

    pub fn adjust(&mut self, n: u32) {
        if n >= self.current {
            self.current *= 2;
        } else {
            self.current = n;
        }

        if self.current > 8 {
            self.current = 8;
        }

        if self.current == 0 {
            self.current = 1;
        }
    }

    pub fn alloc(&self) -> Vec<Buffer> {
        (0..self.current).map(|_| Buffer::new()).collect()
    }
}

impl Default for AllocStrategy {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ReadVReader<R> {
    reader: R,
    alloc: AllocStrategy,
}

impl<R: AsyncRead + Unpin> ReadVReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            alloc: AllocStrategy::new(),
        }
    }

    pub fn alloc_strategy(&self) -> &AllocStrategy {
        &self.alloc
    }

    pub async fn read_multi(&mut self, max_bytes: usize) -> Result<MultiBuffer> {
        let mut buf = vec![0u8; max_bytes];
        let n = self.reader.read(&mut buf).await.map_err(Error::Io)?;
        buf.truncate(n);
        Ok(MultiBuffer::from_bytes(&buf))
    }
}

#[async_trait]
impl<R: AsyncRead + Send + Sync + Unpin> Reader for ReadVReader<R> {
    async fn read_multi_buffer(&mut self) -> Result<MultiBuffer> {
        let current_count = self.alloc.current() as usize;
        let mut mb = MultiBuffer::new();
        let mut chunk = [0u8; 8192];

        for _ in 0..current_count {
            let n = self.reader.read(&mut chunk).await.map_err(Error::Io)?;
            if n == 0 {
                break;
            }
            let mut b = Buffer::with_capacity(n);
            let _ = b.write(&chunk[..n]);
            mb.push(b);
            if n < 8192 {
                break;
            }
        }

        if mb.is_empty() {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "eof",
            )));
        }

        self.alloc.adjust(mb.buffers().len() as u32);
        Ok(mb)
    }
}
