// Module: proxy\shadowsocks_2022\client.rs
// 1:1 Rust implementation corresponding to Go proxy\shadowsocks_2022\client.go

use async_trait::async_trait;

use crate::common::errors::{Error, Result};
use crate::common::net::{BoxStream, Destination};
use crate::common::protocol::SessionContext;
use crate::features::outbound::OutboundHandler;
use crate::proxy::shadowsocks_2022::config::Shadowsocks2022Config;
use crate::proxy::shadowsocks_2022::protocol::{HEADER_TYPE_SERVER, SessionHeader};
use crate::transport::internet::TcpDialer;

pub struct Client {
    pub tag: String,
    pub server_addr: Destination,
    pub config: Shadowsocks2022Config,
}

impl Client {
    pub fn new(
        tag: impl Into<String>,
        server_addr: Destination,
        config: Shadowsocks2022Config,
    ) -> Self {
        Self {
            tag: tag.into(),
            server_addr,
            config,
        }
    }
}

#[async_trait]
impl OutboundHandler for Client {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn connect(&self, session: &SessionContext) -> Result<BoxStream> {
        let mut stream = TcpDialer::dial(&self.server_addr).await?;

        // 1. Encode Shadowsocks-2022 client session header
        let req_header = SessionHeader::new_client(session.destination.clone());
        let expected_session_id = req_header.session_id;
        req_header.encode(&mut stream).await?;

        // 2. Decode Shadowsocks-2022 server response header
        let resp_header = SessionHeader::decode(&mut stream).await?;
        if resp_header.header_type != HEADER_TYPE_SERVER {
            return Err(Error::Protocol(format!(
                "Invalid server header type: expected {}, got {}",
                HEADER_TYPE_SERVER, resp_header.header_type
            )));
        }

        if resp_header.session_id != expected_session_id {
            return Err(Error::Protocol(format!(
                "Session ID mismatch: expected {}, got {}",
                expected_session_id, resp_header.session_id
            )));
        }

        Ok(stream)
    }
}
