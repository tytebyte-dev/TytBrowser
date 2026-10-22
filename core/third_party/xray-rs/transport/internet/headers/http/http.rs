// Module: transport\internet\headers\http\http.rs
// 1:1 Rust implementation corresponding to Go transport\internet\headers\http\http.go

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};

use super::config_pb::Config;
use super::linked_read_request::read_request;
use crate::common::errors::{Error, Result};
use crate::common::net::BoxStream;
use crate::transport::internet::header::ConnectionAuthenticator;

pub const CRLF: &str = "\r\n";
pub const ENDING: &str = "\r\n\r\n";
pub const MAX_HEADER_LENGTH: usize = 8192;

pub struct HeaderWriter {
    pub header: Option<Vec<u8>>,
}

impl HeaderWriter {
    pub fn new(header: Vec<u8>) -> Self {
        Self {
            header: Some(header),
        }
    }

    pub fn write_to_vec(&mut self, target: &mut Vec<u8>) {
        if let Some(h) = self.header.take() {
            target.extend_from_slice(&h);
        }
    }
}

pub struct HeaderReader {
    pub expected_uri: Vec<String>,
}

impl HeaderReader {
    pub fn new(expected_uri: Vec<String>) -> Self {
        Self { expected_uri }
    }

    pub fn parse_and_validate(&self, raw_header: &[u8]) -> Result<()> {
        let req = read_request(raw_header)?;
        if !self.expected_uri.is_empty() {
            let found = self.expected_uri.iter().any(|u| u == &req.uri);
            if !found {
                return Err(Error::Protocol("Header Mismatch".into()));
            }
        }
        Ok(())
    }
}

pub struct HttpStream<S> {
    inner: S,
    header_to_write: Option<Vec<u8>>,
    written_header: bool,
    read_header: bool,
    header_buf: Vec<u8>,
    prefix_buf: Vec<u8>,
    prefix_pos: usize,
    expected_uris: Vec<String>,
}

impl<S> HttpStream<S> {
    pub fn new_client(inner: S, request_header: Vec<u8>) -> Self {
        Self {
            inner,
            header_to_write: Some(request_header),
            written_header: false,
            read_header: false,
            header_buf: Vec::new(),
            prefix_buf: Vec::new(),
            prefix_pos: 0,
            expected_uris: Vec::new(),
        }
    }

    pub fn new_server(inner: S, response_header: Vec<u8>, expected_uris: Vec<String>) -> Self {
        Self {
            inner,
            header_to_write: Some(response_header),
            written_header: false,
            read_header: false,
            header_buf: Vec::new(),
            prefix_buf: Vec::new(),
            prefix_pos: 0,
            expected_uris,
        }
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for HttpStream<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        // First consume prefix buffer from after the header
        if self.prefix_pos < self.prefix_buf.len() {
            let remaining = &self.prefix_buf[self.prefix_pos..];
            let to_copy = remaining.len().min(buf.remaining());
            buf.put_slice(&remaining[..to_copy]);
            self.prefix_pos += to_copy;
            return Poll::Ready(Ok(()));
        }

        // If header hasn't been read yet, read until \r\n\r\n
        if !self.read_header {
            let mut chunk = [0u8; 1024];
            let mut read_buf = ReadBuf::new(&mut chunk);
            match Pin::new(&mut self.inner).poll_read(cx, &mut read_buf) {
                Poll::Ready(Ok(())) => {
                    let filled = read_buf.filled();
                    if filled.is_empty() {
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "EOF before HTTP header completed",
                        )));
                    }
                    self.header_buf.extend_from_slice(filled);
                    if let Some(pos) = self.header_buf.windows(4).position(|w| w == b"\r\n\r\n") {
                        let header_end = pos + 4;
                        let header_bytes = &self.header_buf[..header_end];

                        // Validate request if this is server
                        if !self.expected_uris.is_empty() {
                            let hr = HeaderReader::new(self.expected_uris.clone());
                            if let Err(e) = hr.parse_and_validate(header_bytes) {
                                return Poll::Ready(Err(io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    format!("{:?}", e),
                                )));
                            }
                        }

                        if header_end < self.header_buf.len() {
                            self.prefix_buf = self.header_buf[header_end..].to_vec();
                            self.prefix_pos = 0;
                        }
                        self.read_header = true;
                        self.header_buf.clear();

                        if self.prefix_pos < self.prefix_buf.len() {
                            let remaining = &self.prefix_buf[self.prefix_pos..];
                            let to_copy = remaining.len().min(buf.remaining());
                            buf.put_slice(&remaining[..to_copy]);
                            self.prefix_pos += to_copy;
                        }
                        return Poll::Ready(Ok(()));
                    }
                    if self.header_buf.len() > MAX_HEADER_LENGTH {
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Header too long.",
                        )));
                    }
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }

        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for HttpStream<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        if !self.written_header {
            if let Some(header) = self.header_to_write.take() {
                match Pin::new(&mut self.inner).poll_write(cx, &header) {
                    Poll::Ready(Ok(n)) => {
                        if n < header.len() {
                            self.header_to_write = Some(header[n..].to_vec());
                            return Poll::Pending;
                        }
                        self.written_header = true;
                    }
                    Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                    Poll::Pending => {
                        self.header_to_write = Some(header);
                        return Poll::Pending;
                    }
                }
            } else {
                self.written_header = true;
            }
        }

        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

pub struct Authenticator {
    config: Config,
}

impl Authenticator {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn format_request(&self) -> Vec<u8> {
        if let Some(req) = &self.config.request {
            let method = req.get_method_value();
            let uri = req.pick_uri();
            let version = req.get_full_version();
            let mut s = format!("{} {} {}\r\n", method, uri, version);
            for h in req.pick_headers() {
                s.push_str(&h);
                s.push_str(CRLF);
            }
            s.push_str(CRLF);
            s.into_bytes()
        } else {
            b"GET / HTTP/1.1\r\n\r\n".to_vec()
        }
    }

    pub fn format_response(&self) -> Vec<u8> {
        if let Some(resp) = &self.config.response {
            let version = resp.get_full_version();
            let code = resp.get_status_code();
            let reason = resp.get_status_reason();
            let mut s = format!("{} {} {}\r\n", version, code, reason);
            for h in resp.pick_headers() {
                s.push_str(&h);
                s.push_str(CRLF);
            }
            s.push_str(CRLF);
            s.into_bytes()
        } else {
            b"HTTP/1.1 200 OK\r\n\r\n".to_vec()
        }
    }

    pub fn client_stream<S: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static>(
        &self,
        stream: S,
    ) -> BoxStream {
        let req_header = self.format_request();
        Box::pin(HttpStream::new_client(stream, req_header))
    }

    pub fn server_stream<S: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static>(
        &self,
        stream: S,
    ) -> BoxStream {
        let resp_header = self.format_response();
        let expected_uris = self
            .config
            .request
            .as_ref()
            .map(|r| r.uri.clone())
            .unwrap_or_default();
        Box::pin(HttpStream::new_server(stream, resp_header, expected_uris))
    }
}

impl ConnectionAuthenticator for Authenticator {
    fn client(&self, stream: BoxStream) -> BoxStream {
        self.client_stream(stream)
    }

    fn server(&self, stream: BoxStream) -> BoxStream {
        self.server_stream(stream)
    }
}

pub fn new_authenticator(config: Config) -> Authenticator {
    Authenticator::new(config)
}

pub struct HttpHeaderObfuscator;

impl HttpHeaderObfuscator {
    pub async fn client_handshake<S: AsyncRead + AsyncWrite + Unpin>(
        stream: &mut S,
        req_cfg: &super::config_pb::RequestConfig,
        host: &str,
    ) -> Result<()> {
        let req = req_cfg.format_request(host);
        stream.write_all(req.as_bytes()).await.map_err(Error::Io)?;
        Ok(())
    }

    pub async fn server_read_request<R: AsyncRead + Unpin>(
        reader: &mut R,
    ) -> Result<(String, Vec<u8>)> {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 1];
        while buf.len() < MAX_HEADER_LENGTH {
            let n = reader.read(&mut chunk).await.map_err(Error::Io)?;
            if n == 0 {
                return Err(Error::Closed);
            }
            buf.push(chunk[0]);
            if buf.ends_with(b"\r\n\r\n") {
                let header = String::from_utf8(buf)
                    .map_err(|_| Error::Protocol("invalid utf8 header".into()))?;
                let mut trailing = Vec::new();
                let mut rest = [0u8; 1024];
                while let Ok(n) = reader.read(&mut rest).await {
                    if n == 0 {
                        break;
                    }
                    trailing.extend_from_slice(&rest[..n]);
                }
                return Ok((header, trailing));
            }
        }
        Err(Error::Protocol("header too long".into()))
    }
}
