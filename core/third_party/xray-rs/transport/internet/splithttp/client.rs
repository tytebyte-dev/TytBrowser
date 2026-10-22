// Module: transport\internet\splithttp\client.rs
// 1:1 Rust implementation corresponding to Go transport\internet\splithttp\client.go

use async_trait::async_trait;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

use super::common::*;
use super::config::SplitHttpConfig;
use crate::common::errors::{Error, Result};
use crate::common::net::Destination;
use crate::transport::internet::system_dialer::SystemDialer;

#[async_trait]
pub trait DialerClient: Send + Sync {
    fn is_closed(&self) -> bool;

    async fn open_stream(
        &self,
        url: &str,
        session_id: &str,
        body: Option<Pin<Box<dyn AsyncRead + Send + Sync>>>,
        upload_only: bool,
    ) -> Result<(
        Pin<Box<dyn AsyncRead + Send + Sync>>,
        Option<SocketAddr>,
        Option<SocketAddr>,
    )>;

    async fn post_packet(
        &self,
        url: &str,
        session_id: &str,
        seq_str: &str,
        payload: Vec<u8>,
    ) -> Result<()>;
}

pub struct DefaultDialerClient {
    pub transport_config: SplitHttpConfig,
    pub dest: Destination,
    pub http_version: String,
    pub closed: Arc<AtomicBool>,
}

impl DefaultDialerClient {
    pub fn new(
        transport_config: SplitHttpConfig,
        dest: Destination,
        http_version: impl Into<String>,
    ) -> Self {
        Self {
            transport_config,
            dest,
            http_version: http_version.into(),
            closed: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait]
impl DialerClient for DefaultDialerClient {
    fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    async fn open_stream(
        &self,
        url: &str,
        session_id: &str,
        _body: Option<Pin<Box<dyn AsyncRead + Send + Sync>>>,
        upload_only: bool,
    ) -> Result<(
        Pin<Box<dyn AsyncRead + Send + Sync>>,
        Option<SocketAddr>,
        Option<SocketAddr>,
    )> {
        let mut stream = SystemDialer::dial_tcp(&self.dest).await?;
        let local_addr = stream.local_addr().ok();
        let remote_addr = stream.peer_addr().ok();

        let method = if upload_only {
            self.transport_config.get_normalized_uplink_http_method()
        } else {
            "GET"
        };

        let norm_path = self.transport_config.get_normalized_path();
        let target_path = if !session_id.is_empty()
            && self.transport_config.get_normalized_session_placement() == PLACEMENT_PATH
        {
            SplitHttpConfig::append_to_path(&norm_path, session_id)
        } else {
            norm_path
        };

        let host = if !self.transport_config.host.is_empty() {
            &self.transport_config.host
        } else {
            url
        };

        let mut req = format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nAccept-Encoding: identity\r\n",
            method, target_path, host
        );
        if !session_id.is_empty()
            && self.transport_config.get_normalized_session_placement() == PLACEMENT_HEADER
        {
            req.push_str(&format!(
                "{}: {}\r\n",
                self.transport_config.get_normalized_session_key(),
                session_id
            ));
        }

        for (k, v) in &self.transport_config.headers {
            req.push_str(&format!("{}: {}\r\n", k, v));
        }
        req.push_str("\r\n");

        stream.write_all(req.as_bytes()).await?;
        stream.flush().await?;

        // Read response status
        let mut buf = [0u8; 1024];
        let mut n_read = 0;
        loop {
            let n = stream.read(&mut buf[n_read..]).await?;
            if n == 0 {
                return Err(Error::Closed);
            }
            n_read += n;
            if let Some(pos) = buf[..n_read].windows(4).position(|w| w == b"\r\n\r\n") {
                let header_str = String::from_utf8_lossy(&buf[..pos]);
                let status_line = header_str.lines().next().unwrap_or("");
                if !status_line.contains("200") {
                    return Err(Error::Protocol(format!(
                        "unexpected HTTP status: {}",
                        status_line
                    )));
                }
                let remainder = buf[pos + 4..n_read].to_vec();
                let combined_reader: Pin<Box<dyn AsyncRead + Send + Sync>> =
                    if !remainder.is_empty() {
                        Box::pin(tokio::io::AsyncReadExt::chain(
                            std::io::Cursor::new(remainder),
                            stream,
                        ))
                    } else {
                        Box::pin(stream)
                    };
                return Ok((combined_reader, local_addr, remote_addr));
            }
            if n_read >= buf.len() {
                return Err(Error::Protocol("HTTP response headers too long".into()));
            }
        }
    }

    async fn post_packet(
        &self,
        _url: &str,
        session_id: &str,
        seq_str: &str,
        payload: Vec<u8>,
    ) -> Result<()> {
        let mut stream = SystemDialer::dial_tcp(&self.dest).await?;
        let norm_path = self.transport_config.get_normalized_path();
        let target_path = if !session_id.is_empty()
            && self.transport_config.get_normalized_session_placement() == PLACEMENT_PATH
        {
            let p1 = SplitHttpConfig::append_to_path(&norm_path, session_id);
            if !seq_str.is_empty()
                && self.transport_config.get_normalized_seq_placement() == PLACEMENT_PATH
            {
                SplitHttpConfig::append_to_path(&p1, seq_str)
            } else {
                p1
            }
        } else {
            norm_path
        };

        let method = self.transport_config.get_normalized_uplink_http_method();
        let host = if !self.transport_config.host.is_empty() {
            &self.transport_config.host
        } else {
            "localhost"
        };

        let mut req = format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nContent-Length: {}\r\n",
            method,
            target_path,
            host,
            payload.len()
        );

        if !session_id.is_empty()
            && self.transport_config.get_normalized_session_placement() == PLACEMENT_HEADER
        {
            req.push_str(&format!(
                "{}: {}\r\n",
                self.transport_config.get_normalized_session_key(),
                session_id
            ));
        }
        if !seq_str.is_empty()
            && self.transport_config.get_normalized_seq_placement() == PLACEMENT_HEADER
        {
            req.push_str(&format!(
                "{}: {}\r\n",
                self.transport_config.get_normalized_seq_key(),
                seq_str
            ));
        }

        for (k, v) in &self.transport_config.headers {
            req.push_str(&format!("{}: {}\r\n", k, v));
        }
        req.push_str("\r\n");

        stream.write_all(req.as_bytes()).await?;
        if !payload.is_empty() {
            stream.write_all(&payload).await?;
        }
        stream.flush().await?;

        // Read 200 response
        let mut buf = [0u8; 512];
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            return Err(Error::Closed);
        }
        let resp = String::from_utf8_lossy(&buf[..n]);
        if !resp.starts_with("HTTP/1.1 200") && !resp.starts_with("HTTP/1.0 200") {
            return Err(Error::Protocol(format!(
                "post packet failed with response: {}",
                resp.lines().next().unwrap_or("")
            )));
        }
        Ok(())
    }
}

// Client formatter used by scenarios and quick formatting
pub struct SplitHttpClient {
    pub config: SplitHttpConfig,
    pub session_id: String,
}

impl SplitHttpClient {
    pub fn new(config: SplitHttpConfig, session_id: impl Into<String>) -> Self {
        Self {
            config,
            session_id: session_id.into(),
        }
    }

    pub fn format_download_request(&self) -> String {
        let mut req = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nAccept-Encoding: identity\r\nX-Session-Id: {}\r\n",
            self.config.path, self.config.host, self.session_id
        );
        for (k, v) in &self.config.headers {
            req.push_str(&format!("{}: {}\r\n", k, v));
        }
        req.push_str("\r\n");
        req
    }

    pub fn format_upload_request(&self, payload_len: usize) -> String {
        let mut req = format!(
            "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Length: {}\r\nX-Session-Id: {}\r\n",
            self.config.path, self.config.host, payload_len, self.session_id
        );
        for (k, v) in &self.config.headers {
            req.push_str(&format!("{}: {}\r\n", k, v));
        }
        req.push_str("\r\n");
        req
    }
}
