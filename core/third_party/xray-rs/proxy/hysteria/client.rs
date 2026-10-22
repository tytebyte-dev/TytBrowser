// Module: proxy\hysteria\client.rs
// 1:1 Rust implementation corresponding to Go proxy\hysteria\client.go

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

use super::config::HysteriaConfig;
use super::protocol::TcpRequest;
use crate::common::errors::{Error, Result};
use crate::common::net::{BoxStream, Destination};
use crate::common::protocol::SessionContext;
use crate::features::outbound::OutboundHandler;
use crate::transport::internet::system_dialer::SystemDialer;

pub struct HysteriaClient {
    pub tag: String,
    pub server: Destination,
    pub config: HysteriaConfig,
}

impl HysteriaClient {
    pub fn new(tag: impl Into<String>, server: Destination, config: HysteriaConfig) -> Self {
        Self {
            tag: tag.into(),
            server,
            config,
        }
    }
}

#[async_trait]
impl OutboundHandler for HysteriaClient {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn connect(&self, session: &SessionContext) -> Result<BoxStream> {
        let mut stream = SystemDialer::dial_tcp(&self.server).await?;

        let req = TcpRequest::new(session.destination.to_string(), 0);
        let frame = req.encode();

        stream.write_all(&frame).await.map_err(Error::Io)?;
        stream.flush().await.map_err(Error::Io)?;

        Ok(Box::pin(stream))
    }
}
