// Module: testing\mocks\outbound.rs
// Mock outbound handler implementing OutboundHandler

use crate::common::errors::Result;
use crate::common::net::BoxStream;
use crate::common::protocol::SessionContext;
use crate::features::outbound::OutboundHandler;
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct MockOutboundHandler {
    tag: String,
    invocations: Arc<AtomicUsize>,
}

impl MockOutboundHandler {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            invocations: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn count_invocation(&self) {
        self.invocations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn invocations(&self) -> usize {
        self.invocations.load(Ordering::Relaxed)
    }
}

#[async_trait]
impl OutboundHandler for MockOutboundHandler {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn connect(&self, _session: &SessionContext) -> Result<BoxStream> {
        self.count_invocation();
        let (client, _server) = tokio::io::duplex(4096);
        Ok(Box::pin(client))
    }
}
