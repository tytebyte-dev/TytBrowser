// Module: proxy\shadowsocks\client.rs
// 1:1 Rust implementation corresponding to Go proxy\shadowsocks\client.go

use async_trait::async_trait;

use crate::common::errors::Result;
use crate::common::net::{BoxStream, Destination};
use crate::common::protocol::SessionContext;
use crate::features::outbound::OutboundHandler;
use crate::proxy::shadowsocks::protocol::write_target_address;
use crate::transport::internet::TcpDialer;

pub struct Client {
    tag: String,
    server_addr: Destination,
    method: String,
    password: String,
}

impl Client {
    pub fn new(
        tag: impl Into<String>,
        server_addr: Destination,
        method: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            tag: tag.into(),
            server_addr,
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
impl OutboundHandler for Client {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn connect(&self, session: &SessionContext) -> Result<BoxStream> {
        let mut stream = TcpDialer::dial(&self.server_addr).await?;
        write_target_address(&mut stream, &session.destination).await?;
        Ok(stream)
    }
}
