// Module: common\singbridge\dialer.rs
// 1:1 Rust implementation corresponding to Go common\singbridge\dialer.go

use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;

use super::destination::{Socksaddr, to_destination, to_network};
use crate::common::ctx::Context;
use crate::common::errors::{Error, Result};
use crate::common::net::BoxStream;
use crate::common::protocol::SessionContext;
use crate::features::outbound::OutboundHandler;
use crate::transport::internet::dialer::Dialer as InternetDialer;

/// SingDialerTrait abstracts dialing functionality for singbridge.
/// 1:1 corresponding to sagernet/sing/common/network.Dialer.
#[async_trait]
pub trait SingDialerTrait: Send + Sync {
    async fn dial_context(
        &self,
        ctx: &Context,
        network: &str,
        destination: Socksaddr,
    ) -> Result<BoxStream>;
    async fn listen_packet(&self, ctx: &Context, destination: Socksaddr) -> Result<()>;
}

/// XrayDialer wraps transport internet.Dialer into SingDialerTrait.
/// 1:1 corresponding to XrayDialer in dialer.go.
pub struct XrayDialer {
    dialer: Arc<dyn InternetDialer>,
}

impl XrayDialer {
    pub fn new(dialer: Arc<dyn InternetDialer>) -> Self {
        Self { dialer }
    }
}

#[async_trait]
impl SingDialerTrait for XrayDialer {
    async fn dial_context(
        &self,
        _ctx: &Context,
        network: &str,
        destination: Socksaddr,
    ) -> Result<BoxStream> {
        let dest = to_destination(&destination, to_network(network));
        self.dialer.dial(&dest).await
    }

    async fn listen_packet(&self, _ctx: &Context, _destination: Socksaddr) -> Result<()> {
        Err(Error::Other("listen packet not supported on dialer".into()))
    }
}

/// XrayOutboundDialer routes outgoing connections through an Xray outbound handler.
/// 1:1 corresponding to XrayOutboundDialer in dialer.go.
pub struct XrayOutboundDialer {
    outbound: Arc<dyn OutboundHandler>,
    dialer: Option<Arc<dyn InternetDialer>>,
}

impl XrayOutboundDialer {
    pub fn new(
        outbound: Arc<dyn OutboundHandler>,
        dialer: Option<Arc<dyn InternetDialer>>,
    ) -> Self {
        Self { outbound, dialer }
    }

    pub fn dialer(&self) -> Option<Arc<dyn InternetDialer>> {
        self.dialer.clone()
    }
}

#[async_trait]
impl SingDialerTrait for XrayOutboundDialer {
    async fn dial_context(
        &self,
        _ctx: &Context,
        network: &str,
        destination: Socksaddr,
    ) -> Result<BoxStream> {
        let dest = to_destination(&destination, to_network(network));
        let session = SessionContext::new("singbridge", dest);
        self.outbound.connect(&session).await
    }

    async fn listen_packet(&self, _ctx: &Context, _destination: Socksaddr) -> Result<()> {
        Err(Error::Other(
            "listen packet not supported on outbound dialer".into(),
        ))
    }
}

/// Backwards compatibility struct
pub struct SingDialer;

impl SingDialer {
    pub async fn dial(&self, addr: SocketAddr) -> Result<BoxStream> {
        let stream = tokio::net::TcpStream::connect(addr)
            .await
            .map_err(Error::Io)?;
        Ok(Box::pin(stream))
    }
}

pub async fn dial_singbox_bridge(addr: SocketAddr) -> Result<BoxStream> {
    let stream = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(Error::Io)?;
    Ok(Box::pin(stream))
}
