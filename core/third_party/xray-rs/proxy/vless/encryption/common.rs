// Module: proxy\vless\encryption\common.rs
// 1:1 Rust implementation corresponding to Go proxy\vless\encryption\common.go

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes128Gcm, Nonce as AesNonce};
use chacha20poly1305::{ChaCha20Poly1305, Nonce as ChaChaNonce};
use rand::Rng;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use crate::common::errors::{Error, Result};

pub const ENCRYPTION_VERSION: u8 = 0x01;
pub const MAX_NONCE: [u8; 12] = [0xFF; 12];
pub const MAX_CHUNK_SIZE: usize = 8192;
pub const HEADER_LEN: usize = 5;
pub const TAG_LEN: usize = 16;

pub fn increase_nonce(nonce: &mut [u8; 12]) {
    for i in 0..12 {
        let idx = 11 - i;
        nonce[idx] = nonce[idx].wrapping_add(1);
        if nonce[idx] != 0 {
            break;
        }
    }
}

pub fn encode_header(h: &mut [u8], l: usize) {
    h[0] = 23;
    h[1] = 3;
    h[2] = 3;
    h[3] = (l >> 8) as u8;
    h[4] = l as u8;
}

pub fn decode_header(h: &[u8]) -> Result<usize> {
    if h.len() < HEADER_LEN {
        return Err(Error::Protocol("Header too short".into()));
    }
    if h[0] != 23 || h[1] != 3 || h[2] != 3 {
        return Err(Error::Protocol(format!(
            "Invalid record type in header: {:?}",
            &h[..3]
        )));
    }
    let l = ((h[3] as usize) << 8) | (h[4] as usize);
    if !(17..=16640).contains(&l) {
        return Err(Error::Protocol(format!("Invalid header length: {}", l)));
    }
    Ok(l)
}

pub enum VlessCipher {
    AesGcm(Aes128Gcm),
    ChaCha(ChaCha20Poly1305),
}

pub struct VlessAead {
    cipher: VlessCipher,
    pub nonce: [u8; 12],
}

impl VlessAead {
    pub fn new(key: &[u8; 32], use_aes: bool) -> Self {
        let cipher = if use_aes {
            let k = aes_gcm::Key::<Aes128Gcm>::from_slice(&key[..16]);
            VlessCipher::AesGcm(Aes128Gcm::new(k))
        } else {
            let k = chacha20poly1305::Key::from_slice(key);
            VlessCipher::ChaCha(ChaCha20Poly1305::new(k))
        };
        Self {
            cipher,
            nonce: [0u8; 12],
        }
    }

    pub fn seal(&mut self, plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
        increase_nonce(&mut self.nonce);
        let payload = Payload {
            msg: plaintext,
            aad,
        };
        match &self.cipher {
            VlessCipher::AesGcm(c) => {
                let n = AesNonce::from_slice(&self.nonce);
                c.encrypt(n, payload)
                    .map_err(|e| Error::Crypto(format!("AES-GCM seal error: {}", e)))
            }
            VlessCipher::ChaCha(c) => {
                let n = ChaChaNonce::from_slice(&self.nonce);
                c.encrypt(n, payload)
                    .map_err(|e| Error::Crypto(format!("ChaCha seal error: {}", e)))
            }
        }
    }

    pub fn open(&mut self, ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
        increase_nonce(&mut self.nonce);
        let payload = Payload {
            msg: ciphertext,
            aad,
        };
        match &self.cipher {
            VlessCipher::AesGcm(c) => {
                let n = AesNonce::from_slice(&self.nonce);
                c.decrypt(n, payload)
                    .map_err(|e| Error::Crypto(format!("AES-GCM open error: {}", e)))
            }
            VlessCipher::ChaCha(c) => {
                let n = ChaChaNonce::from_slice(&self.nonce);
                c.decrypt(n, payload)
                    .map_err(|e| Error::Crypto(format!("ChaCha open error: {}", e)))
            }
        }
    }
}

pub fn parse_padding(padding: &str) -> Result<(Vec<[i32; 3]>, Vec<[i32; 3]>)> {
    if padding.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let mut padding_lens = Vec::new();
    let mut padding_gaps = Vec::new();

    let parts: Vec<&str> = padding.split('.').collect();
    for (i, s) in parts.iter().enumerate() {
        let x: Vec<&str> = s.split('-').collect();
        if x.len() < 3 || x[0].is_empty() || x[1].is_empty() || x[2].is_empty() {
            return Err(Error::Config(format!("Invalid padding parameter: {}", s)));
        }
        let y0: i32 = x[0]
            .parse()
            .map_err(|_| Error::Config("Invalid number".into()))?;
        let y1: i32 = x[1]
            .parse()
            .map_err(|_| Error::Config("Invalid number".into()))?;
        let y2: i32 = x[2]
            .parse()
            .map_err(|_| Error::Config("Invalid number".into()))?;

        if i % 2 == 0 {
            padding_lens.push([y0, y1, y2]);
        } else {
            padding_gaps.push([y0, y1, y2]);
        }
    }

    Ok((padding_lens, padding_gaps))
}

pub fn create_padding(
    padding_lens: &[[i32; 3]],
    padding_gaps: &[[i32; 3]],
) -> (usize, Vec<usize>, Vec<Duration>) {
    let mut rng = rand::thread_rng();
    let default_lens = [[100, 111, 1111], [50, 0, 3333]];
    let default_gaps = [[75, 0, 111]];

    let lens_to_use = if padding_lens.is_empty() {
        &default_lens[..]
    } else {
        padding_lens
    };
    let gaps_to_use = if padding_gaps.is_empty() {
        &default_gaps[..]
    } else {
        padding_gaps
    };

    let mut total_len = 0;
    let mut lens = Vec::new();
    let mut gaps = Vec::new();

    for y in lens_to_use {
        let mut l = 0;
        let prob = rng.gen_range(0..=100);
        if y[0] >= prob {
            let min_v = y[1].min(y[2]);
            let max_v = y[1].max(y[2]);
            l = if max_v > min_v {
                rng.gen_range(min_v..=max_v) as usize
            } else {
                min_v as usize
            };
        }
        lens.push(l);
        total_len += l;
    }

    for y in gaps_to_use {
        let mut g = 0;
        let prob = rng.gen_range(0..=100);
        if y[0] >= prob {
            let min_v = y[1].min(y[2]);
            let max_v = y[1].max(y[2]);
            g = if max_v > min_v {
                rng.gen_range(min_v..=max_v) as u64
            } else {
                min_v as u64
            };
        }
        gaps.push(Duration::from_millis(g));
    }

    (total_len, lens, gaps)
}

pub struct CommonConn<S> {
    pub inner: S,
    pub aead_write: VlessAead,
    pub aead_read: VlessAead,
    read_buf: Vec<u8>,
    write_buf: Vec<u8>,
}

impl<S> CommonConn<S> {
    pub fn new(inner: S, key: &[u8; 32], use_aes: bool) -> Self {
        Self {
            inner,
            aead_write: VlessAead::new(key, use_aes),
            aead_read: VlessAead::new(key, use_aes),
            read_buf: Vec::with_capacity(MAX_CHUNK_SIZE),
            write_buf: Vec::with_capacity(MAX_CHUNK_SIZE + HEADER_LEN + TAG_LEN),
        }
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for CommonConn<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();

        if !this.read_buf.is_empty() {
            let to_write = std::cmp::min(buf.remaining(), this.read_buf.len());
            buf.put_slice(&this.read_buf[..to_write]);
            this.read_buf.drain(..to_write);
            return Poll::Ready(Ok(()));
        }

        // Read encrypted chunk: 5 bytes header + ciphertext
        let mut raw_chunk = [0u8; MAX_CHUNK_SIZE + HEADER_LEN + TAG_LEN];
        let mut raw_buf = ReadBuf::new(&mut raw_chunk);
        match Pin::new(&mut this.inner).poll_read(cx, &mut raw_buf) {
            Poll::Ready(Ok(())) => {
                let filled = raw_buf.filled();
                if filled.is_empty() {
                    return Poll::Ready(Ok(()));
                }

                if filled.len() < HEADER_LEN + TAG_LEN {
                    return Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "Truncated encrypted packet",
                    )));
                }

                let l = match decode_header(&filled[..HEADER_LEN]) {
                    Ok(len) => len,
                    Err(e) => {
                        return Poll::Ready(Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            e.to_string(),
                        )));
                    }
                };

                let cipher_end = (HEADER_LEN + l).min(filled.len());
                let ciphertext = &filled[HEADER_LEN..cipher_end];
                let header = &filled[..HEADER_LEN];

                let plaintext = match this.aead_read.open(ciphertext, header) {
                    Ok(pt) => pt,
                    Err(e) => {
                        return Poll::Ready(Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            e.to_string(),
                        )));
                    }
                };

                let to_write = std::cmp::min(buf.remaining(), plaintext.len());
                buf.put_slice(&plaintext[..to_write]);
                if to_write < plaintext.len() {
                    this.read_buf.extend_from_slice(&plaintext[to_write..]);
                }

                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for CommonConn<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let this = self.get_mut();

        // Flush any pending write data
        while !this.write_buf.is_empty() {
            match Pin::new(&mut this.inner).poll_write(cx, &this.write_buf) {
                Poll::Ready(Ok(n)) => {
                    this.write_buf.drain(..n);
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }

        let chunk_size = std::cmp::min(buf.len(), MAX_CHUNK_SIZE);
        let plaintext_chunk = &buf[..chunk_size];

        let ciphertext_len = chunk_size + TAG_LEN;
        let mut header = [0u8; HEADER_LEN];
        encode_header(&mut header, ciphertext_len);

        let ciphertext = match this.aead_write.seal(plaintext_chunk, &header) {
            Ok(ct) => ct,
            Err(e) => {
                return Poll::Ready(Err(std::io::Error::other(e.to_string())));
            }
        };

        let mut frame = Vec::with_capacity(HEADER_LEN + ciphertext.len());
        frame.extend_from_slice(&header);
        frame.extend_from_slice(&ciphertext);

        match Pin::new(&mut this.inner).poll_write(cx, &frame) {
            Poll::Ready(Ok(n)) => {
                if n < frame.len() {
                    this.write_buf.extend_from_slice(&frame[n..]);
                }
                Poll::Ready(Ok(chunk_size))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();
        while !this.write_buf.is_empty() {
            match Pin::new(&mut this.inner).poll_write(cx, &this.write_buf) {
                Poll::Ready(Ok(n)) => {
                    this.write_buf.drain(..n);
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }
        Pin::new(&mut this.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}
