// Module: transport\internet\finalmask\header\custom\tcp.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\header\custom\tcp.go

use rand::Rng;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};

use super::config::{TCPConfig, TCPSequence};
use crate::common::errors::{Error, Result};

pub const TCP_CUSTOM_MAGIC: &[u8] = b"CUSTOM-TCP";

pub async fn read_sequence<R: AsyncRead + Unpin>(
    reader: &mut R,
    sequence: &TCPSequence,
) -> Result<()> {
    for item in &sequence.sequence {
        let length = (item.rand as usize).max(item.packet.len());
        if length == 0 {
            continue;
        }
        let mut buf = vec![0u8; length];
        reader.read_exact(&mut buf).await.map_err(Error::Io)?;

        if !item.packet.is_empty() && buf != item.packet {
            return Err(Error::Protocol("TCP sequence mismatch".into()));
        }
    }
    Ok(())
}

fn generate_rand_bytes(count: usize, r_min: u8, r_max: u8) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let b = if r_min == r_max {
            r_min
        } else {
            rng.gen_range(r_min..=r_max)
        };
        out.push(b);
    }
    out
}

fn generate_delay(min_d: u64, max_d: u64) -> u64 {
    if min_d == max_d {
        min_d
    } else {
        rand::thread_rng().gen_range(min_d..=max_d)
    }
}

pub async fn write_sequence<W: AsyncWrite + Unpin>(
    writer: &mut W,
    sequence: &TCPSequence,
) -> Result<()> {
    let mut merged = Vec::new();

    for item in &sequence.sequence {
        if item.delay_max > 0 {
            if !merged.is_empty() {
                writer.write_all(&merged).await.map_err(Error::Io)?;
                writer.flush().await.map_err(Error::Io)?;
                merged.clear();
            }
            let min_d = item.delay_min.max(0) as u64;
            let max_d = item.delay_max.max(item.delay_min) as u64;
            let delay = generate_delay(min_d, max_d);
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        }

        if item.rand > 0 {
            let r_min = (item.rand_min as u8).min(item.rand_max as u8);
            let r_max = (item.rand_max as u8).max(r_min);
            let rand_bytes = generate_rand_bytes(item.rand as usize, r_min, r_max);
            merged.extend_from_slice(&rand_bytes);
        } else {
            merged.extend_from_slice(&item.packet);
        }
    }

    if !merged.is_empty() {
        writer.write_all(&merged).await.map_err(Error::Io)?;
        writer.flush().await.map_err(Error::Io)?;
    }
    Ok(())
}

/// Client handshake: executes client sequences and verifies server responses
pub async fn client_handshake<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut S,
    config: &TCPConfig,
) -> Result<()> {
    let mut j = 0;
    for client_seq in &config.clients {
        write_sequence(stream, client_seq).await?;
        if j < config.servers.len() {
            read_sequence(stream, &config.servers[j]).await?;
            j += 1;
        }
    }

    while j < config.servers.len() {
        read_sequence(stream, &config.servers[j]).await?;
        j += 1;
    }
    Ok(())
}

/// Server handshake: reads client sequences and executes server sequences
pub async fn server_handshake<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut S,
    config: &TCPConfig,
) -> Result<()> {
    let mut j = 0;
    for (i, client_seq) in config.clients.iter().enumerate() {
        if let Err(e) = read_sequence(stream, client_seq).await {
            if i < config.errors.len() {
                let _ = write_sequence(stream, &config.errors[i]).await;
            }
            return Err(e);
        }

        if j < config.servers.len() {
            write_sequence(stream, &config.servers[j]).await?;
            j += 1;
        }
    }

    while j < config.servers.len() {
        write_sequence(stream, &config.servers[j]).await?;
        j += 1;
    }
    Ok(())
}

pub struct TcpCustomConn<S> {
    stream: S,
    authenticated: bool,
}

impl<S> TcpCustomConn<S> {
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            authenticated: true,
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }

    pub fn into_inner(self) -> S {
        self.stream
    }

    pub fn get_ref(&self) -> &S {
        &self.stream
    }

    pub fn get_mut(&mut self) -> &mut S {
        &mut self.stream
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for TcpCustomConn<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if !self.authenticated {
            return Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "TCP custom header authentication failed",
            )));
        }
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for TcpCustomConn<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        if !self.authenticated {
            return Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "TCP custom header authentication failed",
            )));
        }
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}
