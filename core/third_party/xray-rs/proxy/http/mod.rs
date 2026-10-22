pub mod client;
pub mod config;
pub mod http;
pub mod server;

use crate::common::errors::{Error, Result};
use crate::common::net::{BoxStream, Destination};
use crate::common::protocol::SessionContext;
use crate::features::inbound::{InboundHandler, InboundResult};
use async_trait::async_trait;
use std::net::SocketAddr;
use std::pin::Pin;
use std::str::FromStr;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};

pub struct Server {
    tag: String,
}

impl Server {
    pub fn new(tag: impl Into<String>) -> Self {
        Self { tag: tag.into() }
    }
}

#[async_trait]
impl InboundHandler for Server {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn handle_connection(
        &self,
        mut stream: BoxStream,
        remote_addr: SocketAddr,
    ) -> Result<InboundResult> {
        let mut buf = [0u8; 4096];
        let mut total_read = 0;
        let mut header_len = 0;
        let mut is_complete = false;

        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);

        while total_read < buf.len() {
            let n = stream.read(&mut buf[total_read..]).await?;
            if n == 0 {
                return Err(Error::Closed);
            }
            total_read += n;

            let mut tmp_headers = [httparse::EMPTY_HEADER; 64];
            let mut tmp_req = httparse::Request::new(&mut tmp_headers);
            match tmp_req.parse(&buf[..total_read]) {
                Ok(httparse::Status::Complete(len)) => {
                    header_len = len;
                    is_complete = true;
                    break;
                }
                Ok(httparse::Status::Partial) => continue,
                Err(e) => {
                    return Err(Error::Protocol(format!(
                        "Failed to parse HTTP request: {}",
                        e
                    )));
                }
            }
        }

        if !is_complete {
            return Err(Error::Protocol("Incomplete HTTP request header".into()));
        }

        // Re-parse to get lifetimes
        let _ = req
            .parse(&buf[..total_read])
            .map_err(|e| Error::Protocol(format!("Failed to re-parse HTTP request: {}", e)))?;

        let method = req
            .method
            .ok_or_else(|| Error::Protocol("Missing HTTP method".into()))?;
        let path = req
            .path
            .ok_or_else(|| Error::Protocol("Missing HTTP path".into()))?;

        let (destination, is_connect) = if method.eq_ignore_ascii_case("CONNECT") {
            let dest = Destination::from_str(path)?;
            (dest, true)
        } else {
            let host_header = headers
                .iter()
                .find(|h| h.name.eq_ignore_ascii_case("Host"))
                .and_then(|h| std::str::from_utf8(h.value).ok())
                .ok_or_else(|| {
                    Error::Protocol("Missing Host header in HTTP proxy request".into())
                })?;

            let dest = if host_header.contains(':') {
                Destination::from_str(host_header)?
            } else {
                Destination::from_str(&format!("{}:80", host_header))?
            };
            (dest, false)
        };

        if is_connect {
            let response = b"HTTP/1.1 200 Connection Established\r\n\r\n";
            stream.write_all(response).await?;
            stream.flush().await?;

            let stream: BoxStream = if total_read > header_len {
                Box::pin(PrefixedStream::new(
                    buf[header_len..total_read].to_vec(),
                    stream,
                ))
            } else {
                stream
            };

            let mut session = SessionContext::new(&self.tag, destination);
            session.source = Some(remote_addr);
            Ok(InboundResult { stream, session })
        } else {
            let stream = Box::pin(PrefixedStream::new(buf[..total_read].to_vec(), stream));
            let mut session = SessionContext::new(&self.tag, destination);
            session.source = Some(remote_addr);
            Ok(InboundResult { stream, session })
        }
    }
}

pub struct PrefixedStream {
    prefix: Vec<u8>,
    prefix_pos: usize,
    inner: BoxStream,
}

impl PrefixedStream {
    pub fn new(prefix: Vec<u8>, inner: BoxStream) -> Self {
        Self {
            prefix,
            prefix_pos: 0,
            inner,
        }
    }
}

impl AsyncRead for PrefixedStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if self.prefix_pos < self.prefix.len() {
            let to_write = std::cmp::min(self.prefix.len() - self.prefix_pos, buf.remaining());
            buf.put_slice(&self.prefix[self.prefix_pos..self.prefix_pos + to_write]);
            self.prefix_pos += to_write;
            if buf.remaining() == 0 {
                return Poll::Ready(Ok(()));
            }
            let _ = Pin::new(&mut self.inner).poll_read(cx, buf);
            return Poll::Ready(Ok(()));
        }
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for PrefixedStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}
