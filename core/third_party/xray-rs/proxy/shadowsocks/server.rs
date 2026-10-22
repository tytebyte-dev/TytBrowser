// Module: proxy\shadowsocks\server.rs
// 1:1 Rust implementation corresponding to Go proxy\shadowsocks\server.go

use async_trait::async_trait;
use std::net::SocketAddr;

use crate::common::errors::Result;
use crate::common::net::{BoxStream, Network};
use crate::common::protocol::SessionContext;
use crate::features::inbound::{InboundHandler, InboundResult};
use crate::proxy::shadowsocks::protocol::read_target_address;

pub struct Server {
    tag: String,
    method: String,
    password: String,
}

impl Server {
    pub fn new(
        tag: impl Into<String>,
        method: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            tag: tag.into(),
            method: method.into(),
            password: password.into(),
        }
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

#[async_trait]
impl InboundHandler for Server {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn handle_connection(
        &self,
        mut stream: BoxStream,
        remote_addr: SocketAddr,
    ) -> Result<InboundResult> {
        let dest = read_target_address(&mut stream).await?;

        let mut session = SessionContext::new(&self.tag, dest);
        session.source = Some(remote_addr);
        session.destination.network = Network::Tcp;

        Ok(InboundResult { stream, session })
    }
}
