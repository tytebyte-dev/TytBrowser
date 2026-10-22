// Module: common\crypto\chunk.rs
// 1:1 Rust implementation corresponding to Go common\crypto\chunk.go

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::common::errors::{Error, Result};

pub struct IncreasingNonce {
    nonce: Vec<u8>,
}

impl IncreasingNonce {
    pub fn new(size: usize) -> Self {
        Self {
            nonce: vec![0u8; size],
        }
    }

    pub fn next(&mut self) -> &[u8] {
        for byte in self.nonce.iter_mut() {
            *byte = byte.wrapping_add(1);
            if *byte != 0 {
                break;
            }
        }
        &self.nonce
    }
}

pub struct PlainChunk;

impl PlainChunk {
    pub async fn write_chunk<W: AsyncWrite + Unpin>(writer: &mut W, data: &[u8]) -> Result<()> {
        if data.len() > u16::MAX as usize {
            return Err(Error::BufferOverflow);
        }
        writer.write_u16(data.len() as u16).await?;
        writer.write_all(data).await?;
        writer.flush().await?;
        Ok(())
    }

    pub async fn read_chunk<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Vec<u8>> {
        let len = reader.read_u16().await? as usize;
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf).await?;
        Ok(buf)
    }
}

pub struct AeadChaCha20ChunkWriter<W> {
    writer: W,
    cipher: ChaCha20Poly1305,
    nonce: IncreasingNonce,
}

impl<W: AsyncWrite + Unpin> AeadChaCha20ChunkWriter<W> {
    pub fn new(writer: W, key: &[u8; 32]) -> Self {
        let key_ref = Key::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key_ref);
        Self {
            writer,
            cipher,
            nonce: IncreasingNonce::new(12),
        }
    }

    pub async fn write_chunk(&mut self, data: &[u8]) -> Result<()> {
        let nonce = Nonce::from_slice(self.nonce.next());
        let ciphertext = self
            .cipher
            .encrypt(nonce, data)
            .map_err(|e| Error::Crypto(e.to_string()))?;
        PlainChunk::write_chunk(&mut self.writer, &ciphertext).await
    }
}

pub struct AeadChaCha20ChunkReader<R> {
    reader: R,
    cipher: ChaCha20Poly1305,
    nonce: IncreasingNonce,
}

impl<R: AsyncRead + Unpin> AeadChaCha20ChunkReader<R> {
    pub fn new(reader: R, key: &[u8; 32]) -> Self {
        let key_ref = Key::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key_ref);
        Self {
            reader,
            cipher,
            nonce: IncreasingNonce::new(12),
        }
    }

    pub async fn read_chunk(&mut self) -> Result<Vec<u8>> {
        let ciphertext = PlainChunk::read_chunk(&mut self.reader).await?;
        let nonce = Nonce::from_slice(self.nonce.next());
        self.cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| Error::Crypto(e.to_string()))
    }
}
