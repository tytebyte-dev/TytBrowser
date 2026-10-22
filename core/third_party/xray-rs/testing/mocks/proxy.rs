// Module: testing\mocks\proxy.rs
// Mock proxy inbound and outbound handlers

use crate::common::errors::Result;
use crate::common::net::{Address, BoxStream, Destination};
use crate::common::protocol::SessionContext;
use crate::features::inbound::{InboundHandler, InboundResult};
use async_trait::async_trait;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub use super::outbound::MockOutboundHandler;
pub use super::outbound::MockOutboundHandler as MockProxyHandler;

pub struct MockInboundHandler {
    tag: String,
    invocations: Arc<AtomicUsize>,
}

impl MockInboundHandler {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            invocations: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn invocations(&self) -> usize {
        self.invocations.load(Ordering::Relaxed)
    }
}

#[async_trait]
impl InboundHandler for MockInboundHandler {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn handle_connection(
        &self,
        stream: BoxStream,
        remote_addr: SocketAddr,
    ) -> Result<InboundResult> {
        self.invocations.fetch_add(1, Ordering::Relaxed);
        let dest = Destination::tcp(Address::Ipv4(Ipv4Addr::new(127, 0, 0, 1)), 80);
        let mut session = SessionContext::new(&self.tag, dest);
        session.source = Some(remote_addr);
        Ok(InboundResult { stream, session })
    }
}
