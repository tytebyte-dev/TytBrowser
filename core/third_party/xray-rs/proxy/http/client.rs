// Module: proxy\http\client.rs
// 1:1 Rust implementation corresponding to Go proxy\http\client.go

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::PrefixedStream;
use crate::common::errors::{Error, Result};
use crate::common::net::{BoxStream, Destination};
use crate::common::protocol::SessionContext;
use crate::features::outbound::OutboundHandler;
use crate::transport::internet::system_dialer::SystemDialer;

pub struct HttpClient {
    tag: String,
    server: Destination,
    auth: Option<String>,
}

impl HttpClient {
    pub fn new(tag: impl Into<String>, server: Destination) -> Self {
        Self {
            tag: tag.into(),
            server,
            auth: None,
        }
    }

    pub fn with_auth(
        tag: impl Into<String>,
        server: Destination,
        username: &str,
        password: &str,
    ) -> Self {
        let auth_str = format!("{}:{}", username, password);
        let auth_header = format!("Basic {}", BASE64.encode(auth_str.as_bytes()));
        Self {
            tag: tag.into(),
            server,
            auth: Some(auth_header),
        }
    }
}

#[async_trait]
impl OutboundHandler for HttpClient {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn connect(&self, session: &SessionContext) -> Result<BoxStream> {
        let mut stream = SystemDialer::dial_tcp(&self.server).await?;

        let target_str = session.destination.to_string();
        let mut req = format!(
            "CONNECT {} HTTP/1.1\r\nHost: {}\r\nProxy-Connection: Keep-Alive\r\n",
            target_str, target_str
        );

        if let Some(auth) = &self.auth {
            req.push_str(&format!("Proxy-Authorization: {}\r\n", auth));
        }
        req.push_str("\r\n");

        stream.write_all(req.as_bytes()).await.map_err(Error::Io)?;
        stream.flush().await.map_err(Error::Io)?;

        // Read response
        let mut buf = vec![0u8; 4096];
        let mut total = 0;
        let mut header_len = 0;
        let mut status_code = 0;

        while total < buf.len() {
            let n = stream.read(&mut buf[total..]).await.map_err(Error::Io)?;
            if n == 0 {
                return Err(Error::Closed);
            }
            total += n;

            let mut headers = [httparse::EMPTY_HEADER; 32];
            let mut resp = httparse::Response::new(&mut headers);
            if let Ok(httparse::Status::Complete(hlen)) = resp.parse(&buf[..total]) {
                header_len = hlen;
                status_code = resp.code.unwrap_or(0);
                break;
            }
        }

        if status_code != 200 {
            return Err(Error::Protocol(format!(
                "HTTP proxy server responded with status: {}",
                status_code
            )));
        }

        let boxed_stream: BoxStream = Box::pin(stream);
        if total > header_len {
            let leftover = buf[header_len..total].to_vec();
            Ok(Box::pin(PrefixedStream::new(leftover, boxed_stream)))
        } else {
            Ok(boxed_stream)
        }
    }
}
