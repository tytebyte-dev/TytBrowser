// Module: common\singbridge\handler.rs
// 1:1 Rust implementation corresponding to Go common\singbridge\handler.go

use async_trait::async_trait;
use std::sync::Arc;

use super::destination::{Socksaddr, to_destination};
use crate::common::ctx::Context;
use crate::common::errors::Result;
use crate::common::log::{GeneralMessage, Severity, record};
use crate::common::net::{BoxStream, Network};
use crate::common::protocol::SessionContext;
use crate::features::routing::Dispatcher as RoutingDispatcher;

/// Metadata contains connection routing source and destination addresses.
/// 1:1 corresponding to sagernet/sing/common/metadata.Metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    pub source: Socksaddr,
    pub destination: Socksaddr,
}

impl Metadata {
    pub fn new(source: Socksaddr, destination: Socksaddr) -> Self {
        Self {
            source,
            destination,
        }
    }
}

/// TCPConnectionHandler handles inbound TCP connections.
/// 1:1 corresponding to sagernet/sing/common/network.TCPConnectionHandler.
#[async_trait]
pub trait TcpConnectionHandler: Send + Sync {
    async fn new_connection(
        &self,
        ctx: &Context,
        conn: BoxStream,
        metadata: Metadata,
    ) -> Result<()>;
}

/// UDPConnectionHandler handles inbound UDP packet connections.
/// 1:1 corresponding to sagernet/sing/common/network.UDPConnectionHandler.
#[async_trait]
pub trait UdpConnectionHandler: Send + Sync {
    async fn new_packet_connection(
        &self,
        ctx: &Context,
        conn: BoxStream,
        metadata: Metadata,
    ) -> Result<()>;
}

/// Dispatcher routes singbridge TCP and UDP connections through an Xray routing dispatcher.
/// 1:1 corresponding to Dispatcher in handler.go.
pub struct Dispatcher {
    upstream: Arc<dyn RoutingDispatcher>,
}

impl Dispatcher {
    pub fn new(upstream: Arc<dyn RoutingDispatcher>) -> Self {
        Self { upstream }
    }

    pub fn new_error(&self, _ctx: &Context, err: &crate::common::errors::Error) {
        record(&GeneralMessage {
            severity: Severity::Info,
            content: format!("[singbridge] {}", err),
        });
    }
}

#[async_trait]
impl TcpConnectionHandler for Dispatcher {
    async fn new_connection(
        &self,
        _ctx: &Context,
        conn: BoxStream,
        metadata: Metadata,
    ) -> Result<()> {
        let dest = to_destination(&metadata.destination, Network::Tcp);
        let session = SessionContext::new("singbridge-tcp", dest);
        self.upstream.dispatch(session, conn).await
    }
}

#[async_trait]
impl UdpConnectionHandler for Dispatcher {
    async fn new_packet_connection(
        &self,
        _ctx: &Context,
        conn: BoxStream,
        metadata: Metadata,
    ) -> Result<()> {
        let dest = to_destination(&metadata.destination, Network::Udp);
        let session = SessionContext::new("singbridge-udp", dest);
        self.upstream.dispatch(session, conn).await
    }
}

/// Backwards compatibility trait
#[async_trait]
pub trait SingHandler: Send + Sync {
    async fn handle(&self, stream: BoxStream, session: &SessionContext) -> Result<()>;
}
