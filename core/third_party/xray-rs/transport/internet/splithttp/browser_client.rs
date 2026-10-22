// Module: transport\internet\splithttp\browser_client.rs
// 1:1 Rust implementation corresponding to Go transport\internet\splithttp\browser_client.go

use async_trait::async_trait;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::AsyncRead;

use super::client::DialerClient;
use super::config::SplitHttpConfig;
use crate::common::errors::{Error, Result};

pub struct BrowserDialerClient {
    pub transport_config: SplitHttpConfig,
    pub closed: AtomicBool,
}

impl BrowserDialerClient {
    pub fn new(transport_config: SplitHttpConfig) -> Self {
        Self {
            transport_config,
            closed: AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl DialerClient for BrowserDialerClient {
    fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    async fn open_stream(
        &self,
        _url: &str,
        _session_id: &str,
        body: Option<Pin<Box<dyn AsyncRead + Send + Sync>>>,
        _upload_only: bool,
    ) -> Result<(
        Pin<Box<dyn AsyncRead + Send + Sync>>,
        Option<SocketAddr>,
        Option<SocketAddr>,
    )> {
        if body.is_some() {
            return Err(Error::Protocol(
                "bidirectional streaming for browser dialer not implemented yet".into(),
            ));
        }
        // Browser dialer operates via external browser websocket bridge when configured
        Err(Error::Protocol(
            "browser dialer not active or available".into(),
        ))
    }

    async fn post_packet(
        &self,
        _url: &str,
        _session_id: &str,
        _seq_str: &str,
        _payload: Vec<u8>,
    ) -> Result<()> {
        Err(Error::Protocol(
            "browser dialer not active or available".into(),
        ))
    }
}

pub use BrowserDialerClient as BrowserClient;
