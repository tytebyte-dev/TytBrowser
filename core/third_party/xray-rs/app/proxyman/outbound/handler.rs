// Module: app\proxyman\outbound\handler.rs
// 1:1 Rust implementation corresponding to Go app\proxyman\outbound\handler.go

use async_trait::async_trait;
use std::sync::Arc;

use crate::common::errors::Result;
use crate::common::net::BoxStream;
use crate::common::protocol::SessionContext;
use crate::features::outbound::OutboundHandler;

pub struct DefaultOutboundHandler {
    tag: String,
    inner: Option<Arc<dyn OutboundHandler>>,
    proxy_protocol: Option<u32>,
}

impl DefaultOutboundHandler {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            inner: None,
            proxy_protocol: None,
        }
    }

    pub fn with_inner(tag: impl Into<String>, inner: Arc<dyn OutboundHandler>) -> Self {
        Self {
            tag: tag.into(),
            inner: Some(inner),
            proxy_protocol: None,
        }
    }

    pub fn with_proxy_protocol(mut self, version: u32) -> Self {
        self.proxy_protocol = Some(version);
        self
    }

    pub fn inner(&self) -> Option<Arc<dyn OutboundHandler>> {
        self.inner.clone()
    }
}

#[async_trait]
impl OutboundHandler for DefaultOutboundHandler {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn connect(&self, session: &SessionContext) -> Result<BoxStream> {
        if let Some(inner) = &self.inner {
            return inner.connect(session).await;
        }
        crate::transport::internet::TcpDialer::dial(&session.destination).await
    }
}

pub use DefaultOutboundHandler as Handler;
