// Module: features\routing\dispatcher.rs
// 1:1 Rust implementation corresponding to Go features\routing\dispatcher.go

use crate::common::errors::Result;
use crate::common::net::BoxStream;
use crate::common::protocol::SessionContext;
use crate::features::feature::{Feature, TYPE_DISPATCHER};
use async_trait::async_trait;

#[async_trait]
pub trait Dispatcher: Feature {
    async fn dispatch(&self, session: SessionContext, stream: BoxStream) -> Result<()>;
}

#[async_trait]
pub trait DispatcherFeature: Send + Sync {
    async fn dispatch(&self, inbound_stream: BoxStream, session: SessionContext) -> Result<()>;
}

pub fn dispatcher_type() -> &'static str {
    TYPE_DISPATCHER
}
