// Module: common\crypto\io.rs
// 1:1 Rust implementation corresponding to Go common\crypto\io.go

use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::crypto::StreamCipher;
use crate::common::buf::MultiBuffer;
use crate::common::buf::io::Writer as BufWriter;
use crate::common::errors::{Error, Result};

pub fn xor_buffers(dst: &mut [u8], src: &[u8], key: &[u8]) -> Result<usize> {
    let len = dst.len().min(src.len());
    for i in 0..len {
        dst[i] = src[i] ^ key[i % key.len()];
    }
    Ok(len)
}

pub struct CryptionReader<R, C> {
    reader: R,
    cipher: C,
}

impl<R: AsyncRead + Unpin, C: StreamCipher> CryptionReader<R, C> {
    pub fn new(reader: R, cipher: C) -> Self {
        Self { reader, cipher }
    }

    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = self.reader.read(buf).await.map_err(Error::Io)?;
        if n > 0 {
            self.cipher.decrypt(&mut buf[..n])?;
        }
        Ok(n)
    }
}

pub struct CryptionWriter<W, C> {
    writer: W,
    cipher: C,
}

impl<W: AsyncWrite + Unpin, C: StreamCipher> CryptionWriter<W, C> {
    pub fn new(writer: W, cipher: C) -> Self {
        Self { writer, cipher }
    }

    pub async fn write(&mut self, data: &mut [u8]) -> Result<usize> {
        self.cipher.encrypt(data)?;
        self.writer.write_all(data).await.map_err(Error::Io)?;
        Ok(data.len())
    }
}

#[async_trait]
impl<W: AsyncWrite + Send + Sync + Unpin, C: StreamCipher + Send> BufWriter
    for CryptionWriter<W, C>
{
    async fn write_multi_buffer(&mut self, mut mb: MultiBuffer) -> Result<()> {
        for b in mb.buffers_mut() {
            let slice = b.as_mut_slice();
            self.cipher.encrypt(slice)?;
            self.writer.write_all(slice).await.map_err(Error::Io)?;
        }
        self.writer.flush().await.map_err(Error::Io)?;
        Ok(())
    }
}
