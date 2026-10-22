// Module: features\outbound\outbound.rs
// 1:1 Rust implementation corresponding to Go features\outbound\outbound.go

use crate::common::errors::Result;
use crate::common::net::BoxStream;
use crate::common::protocol::SessionContext;
use crate::features::feature::TYPE_OUTBOUND_MANAGER;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait OutboundHandler: Send + Sync {
    fn tag(&self) -> &str;

    async fn start(&self) -> Result<()> {
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }

    async fn connect(&self, session: &SessionContext) -> Result<BoxStream>;

    fn sender_settings(&self) -> Option<Vec<u8>> {
        None
    }

    fn proxy_settings(&self) -> Option<Vec<u8>> {
        None
    }
}

pub trait HandlerSelector: Send + Sync {
    fn select(&self, tags: &[String]) -> Vec<String>;
}

#[async_trait]
pub trait OutboundManager: Send + Sync {
    async fn get_handler(&self, tag: &str) -> Option<Arc<dyn OutboundHandler>>;
    async fn get_default_handler(&self) -> Option<Arc<dyn OutboundHandler>>;
    async fn add_handler(&self, handler: Arc<dyn OutboundHandler>) -> Result<()>;
    async fn remove_handler(&self, tag: &str) -> Result<()>;
    async fn list_handlers(&self) -> Vec<Arc<dyn OutboundHandler>>;
}

pub fn manager_type() -> &'static str {
    TYPE_OUTBOUND_MANAGER
}
