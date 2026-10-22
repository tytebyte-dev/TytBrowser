pub mod config;
#[path = "config.pb.rs"]
pub mod config_pb;
pub mod dokodemo;
pub mod fakeudp_linux;
pub mod fakeudp_other;

use crate::common::errors::Result;
use crate::common::net::{BoxStream, Destination, Network};
use crate::common::protocol::SessionContext;
use crate::features::inbound::{InboundHandler, InboundResult};
use async_trait::async_trait;
use std::net::SocketAddr;

pub use config::DokodemoConfig;
pub use dokodemo::DokodemoHandler;

pub struct Server {
    tag: String,
    destination: Destination,
    network: Network,
}

impl Server {
    pub fn new(tag: impl Into<String>, destination: Destination, network: Network) -> Self {
        Self {
            tag: tag.into(),
            destination,
            network,
        }
    }
}

#[async_trait]
impl InboundHandler for Server {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn handle_connection(
        &self,
        stream: BoxStream,
        remote_addr: SocketAddr,
    ) -> Result<InboundResult> {
        let mut session = SessionContext::new(&self.tag, self.destination.clone());
        session.source = Some(remote_addr);
        session.destination.network = self.network;

        Ok(InboundResult { stream, session })
    }
}
