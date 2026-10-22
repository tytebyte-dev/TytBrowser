// Module: features\inbound\inbound.rs
// 1:1 Rust implementation corresponding to Go features\inbound\inbound.go

use crate::common::errors::Result;
use crate::common::net::BoxStream;
use crate::common::protocol::SessionContext;
use crate::features::feature::TYPE_INBOUND_MANAGER;
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct InboundResult {
    pub stream: BoxStream,
    pub session: SessionContext,
}

#[async_trait]
pub trait InboundHandler: Send + Sync {
    fn tag(&self) -> &str;

    async fn start(&self) -> Result<()> {
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }

    fn receiver_settings(&self) -> Option<Vec<u8>> {
        None
    }

    fn proxy_settings(&self) -> Option<Vec<u8>> {
        None
    }

    async fn handle_connection(
        &self,
        stream: BoxStream,
        remote_addr: SocketAddr,
    ) -> Result<InboundResult>;
}

#[async_trait]
pub trait InboundManager: Send + Sync {
    async fn get_handler(&self, tag: &str) -> Result<Arc<dyn InboundHandler>>;
    async fn add_handler(&self, handler: Arc<dyn InboundHandler>) -> Result<()>;
    async fn remove_handler(&self, tag: &str) -> Result<()>;
    async fn list_handlers(&self) -> Vec<Arc<dyn InboundHandler>>;
}

pub fn manager_type() -> &'static str {
    TYPE_INBOUND_MANAGER
}
