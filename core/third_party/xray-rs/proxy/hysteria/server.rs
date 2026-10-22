// Module: proxy\hysteria\server.rs
// 1:1 Rust implementation corresponding to Go proxy\hysteria\server.go

use super::config::HysteriaConfig;
use super::protocol::TcpRequest;
use crate::common::errors::Result;
use crate::common::net::{BoxStream, Destination};
use crate::common::protocol::SessionContext;
use crate::features::inbound::{InboundHandler, InboundResult};
use async_trait::async_trait;
use std::net::SocketAddr;
use std::str::FromStr;

pub struct HysteriaServer {
    pub tag: String,
    pub config: HysteriaConfig,
}

impl HysteriaServer {
    pub fn new(tag: impl Into<String>, config: HysteriaConfig) -> Self {
        Self {
            tag: tag.into(),
            config,
        }
    }
}

#[async_trait]
impl InboundHandler for HysteriaServer {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn handle_connection(
        &self,
        mut stream: BoxStream,
        remote_addr: SocketAddr,
    ) -> Result<InboundResult> {
        let req = TcpRequest::decode(&mut stream).await?;
        let dest = Destination::from_str(&req.address)?;

        let mut session = SessionContext::new(&self.tag, dest);
        session.source = Some(remote_addr);

        Ok(InboundResult { stream, session })
    }
}
