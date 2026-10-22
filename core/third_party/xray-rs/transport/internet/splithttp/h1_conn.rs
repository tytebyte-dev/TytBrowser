// Module: transport\internet\splithttp\h1_conn.rs
// 1:1 Rust implementation corresponding to Go transport\internet\splithttp\h1_conn.go

use crate::common::net::BoxStream;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, ReadBuf};

pub struct H1Conn {
    pub unread_responses_count: usize,
    pub stream: BufReader<BoxStream>,
}

impl H1Conn {
    pub fn new(stream: BoxStream) -> Self {
        Self {
            unread_responses_count: 0,
            stream: BufReader::new(stream),
        }
    }

    pub fn inc_unread(&mut self) {
        self.unread_responses_count += 1;
    }

    pub fn dec_unread(&mut self) {
        if self.unread_responses_count > 0 {
            self.unread_responses_count -= 1;
        }
    }

    pub async fn write_all(&mut self, src: &[u8]) -> io::Result<()> {
        self.stream.get_mut().write_all(src).await
    }

    pub async fn flush(&mut self) -> io::Result<()> {
        self.stream.get_mut().flush().await
    }

    pub async fn read_response_status(&mut self) -> io::Result<u16> {
        let mut line = String::new();
        let mut byte = [0u8; 1];
        loop {
            let n = self.stream.read(&mut byte).await?;
            if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "EOF while reading HTTP status",
                ));
            }
            line.push(byte[0] as char);
            if line.ends_with("\r\n") {
                break;
            }
            if line.len() > 1024 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "HTTP status line too long",
                ));
            }
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2
            && let Ok(code) = parts[1].parse::<u16>()
        {
            self.dec_unread();
            return Ok(code);
        }
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid HTTP status line",
        ))
    }
}

impl AsyncRead for H1Conn {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for H1Conn {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(self.stream.get_mut()).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(self.stream.get_mut()).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(self.stream.get_mut()).poll_shutdown(cx)
    }
}
