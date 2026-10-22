// Module: transport\internet\httpupgrade\httpupgrade.rs
// 1:1 Rust implementation corresponding to Go transport\internet\httpupgrade\httpupgrade.go

use crate::common::errors::{Error, Result};
use crate::common::net::BoxStream;
use std::collections::HashMap;
use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const PROTOCOL_NAME: &str = "httpupgrade";

pub struct UpgradedStream {
    inner: BoxStream,
    prefix: Vec<u8>,
    prefix_pos: usize,
}

impl UpgradedStream {
    pub fn new(inner: BoxStream, prefix: Vec<u8>) -> Self {
        Self {
            inner,
            prefix,
            prefix_pos: 0,
        }
    }
}

impl AsyncRead for UpgradedStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        if self.prefix_pos < self.prefix.len() {
            let available = self.prefix.len() - self.prefix_pos;
            let to_read = available.min(buf.remaining());
            buf.put_slice(&self.prefix[self.prefix_pos..self.prefix_pos + to_read]);
            self.prefix_pos += to_read;
            return std::task::Poll::Ready(Ok(()));
        }
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for UpgradedStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

pub struct HttpUpgradeStream;

impl HttpUpgradeStream {
    pub async fn client_handshake(
        host: &str,
        path: &str,
        mut stream: BoxStream,
        headers: &HashMap<String, String>,
    ) -> Result<BoxStream> {
        let mut req = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n",
            path, host
        );
        for (k, v) in headers {
            req.push_str(&format!("{}: {}\r\n", k, v));
        }
        req.push_str("\r\n");
        stream.write_all(req.as_bytes()).await?;
        stream.flush().await?;

        // Read until header termination
        let mut buf = Vec::new();
        let mut temp = [0u8; 1024];
        let header_end;
        loop {
            let n = stream.read(&mut temp).await?;
            if n == 0 {
                return Err(Error::Protocol(
                    "Unexpected EOF reading HTTPUpgrade handshake response".into(),
                ));
            }
            buf.extend_from_slice(&temp[..n]);
            if let Some(pos) = find_header_end(&buf) {
                header_end = pos;
                break;
            }
            if buf.len() > 16 * 1024 {
                return Err(Error::Protocol(
                    "HTTPUpgrade response header too large".into(),
                ));
            }
        }

        let header_str = String::from_utf8_lossy(&buf[..header_end]);
        let header_lower = header_str.to_ascii_lowercase();
        if !header_str.starts_with("HTTP/1.1 101") && !header_str.starts_with("HTTP/1.0 101") {
            return Err(Error::Protocol(format!(
                "HTTPUpgrade server rejected: {}",
                header_str.lines().next().unwrap_or("")
            )));
        }
        if !header_lower.contains("upgrade: websocket")
            || !header_lower.contains("connection: upgrade")
        {
            return Err(Error::Protocol(
                "HTTPUpgrade response missing valid Upgrade/Connection headers".into(),
            ));
        }

        let remaining = buf[header_end..].to_vec();
        let upgraded = UpgradedStream::new(stream, remaining);
        Ok(Box::pin(upgraded))
    }

    pub async fn server_handshake(
        mut stream: BoxStream,
        expected_host: Option<&str>,
        expected_path: Option<&str>,
    ) -> Result<BoxStream> {
        let mut buf = Vec::new();
        let mut temp = [0u8; 1024];
        let header_end;
        loop {
            let n = stream.read(&mut temp).await?;
            if n == 0 {
                return Err(Error::Protocol(
                    "Unexpected EOF reading HTTPUpgrade request".into(),
                ));
            }
            buf.extend_from_slice(&temp[..n]);
            if let Some(pos) = find_header_end(&buf) {
                header_end = pos;
                break;
            }
            if buf.len() > 16 * 1024 {
                return Err(Error::Protocol(
                    "HTTPUpgrade request header too large".into(),
                ));
            }
        }

        let header_str = String::from_utf8_lossy(&buf[..header_end]);
        let header_lower = header_str.to_ascii_lowercase();

        // Check request line
        let first_line = header_str.lines().next().unwrap_or("");
        let mut parts = first_line.split_whitespace();
        let method = parts.next().unwrap_or("");
        let path = parts.next().unwrap_or("");
        if method != "GET" {
            return Err(Error::Protocol(format!("Invalid HTTP method: {}", method)));
        }
        if let Some(exp_path) = expected_path
            && !exp_path.is_empty()
            && path != exp_path
        {
            return Err(Error::Protocol(format!(
                "Path mismatch: got '{}', expected '{}'",
                path, exp_path
            )));
        }
        if let Some(exp_host) = expected_host
            && !exp_host.is_empty()
            && !header_lower.contains(&format!("host: {}", exp_host.to_ascii_lowercase()))
        {
            return Err(Error::Protocol(format!(
                "Host mismatch: expected '{}'",
                exp_host
            )));
        }

        if !header_lower.contains("upgrade: websocket")
            || !header_lower.contains("connection: upgrade")
        {
            return Err(Error::Protocol(
                "Invalid HTTPUpgrade request headers".into(),
            ));
        }

        let resp =
            "HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nUpgrade: websocket\r\n\r\n";
        stream.write_all(resp.as_bytes()).await?;
        stream.flush().await?;

        let remaining = buf[header_end..].to_vec();
        let upgraded = UpgradedStream::new(stream, remaining);
        Ok(Box::pin(upgraded))
    }
}

fn find_header_end(data: &[u8]) -> Option<usize> {
    for i in 0..data.len() {
        if i + 4 <= data.len() && &data[i..i + 4] == b"\r\n\r\n" {
            return Some(i + 4);
        }
        if i + 2 <= data.len() && &data[i..i + 2] == b"\n\n" {
            return Some(i + 2);
        }
    }
    None
}
