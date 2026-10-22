// Module: proxy\shadowsocks_2022\server.rs
// 1:1 Rust implementation corresponding to Go proxy\shadowsocks_2022\server.go

use async_trait::async_trait;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::common::errors::{Error, Result};
use crate::common::net::{BoxStream, Network};
use crate::common::protocol::SessionContext;
use crate::features::inbound::{InboundHandler, InboundResult};
use crate::proxy::shadowsocks_2022::config::Shadowsocks2022Config;
use crate::proxy::shadowsocks_2022::protocol::{HEADER_TYPE_CLIENT, SessionHeader};

pub struct Server {
    pub tag: String,
    pub config: Shadowsocks2022Config,
}

impl Server {
    pub fn new(tag: impl Into<String>, config: Shadowsocks2022Config) -> Self {
        Self {
            tag: tag.into(),
            config,
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
        mut stream: BoxStream,
        remote_addr: SocketAddr,
    ) -> Result<InboundResult> {
        // 1. Read and decode 2022 SessionHeader
        let header = SessionHeader::decode(&mut stream).await?;

        if header.header_type != HEADER_TYPE_CLIENT {
            return Err(Error::Protocol(format!(
                "Expected client header type, got {}",
                header.header_type
            )));
        }

        // 2. Anti-replay timestamp check (window = 300 seconds)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::Protocol(e.to_string()))?
            .as_secs();

        let diff = now.abs_diff(header.timestamp);

        if diff > 300 {
            return Err(Error::AuthFailed(format!(
                "Shadowsocks-2022 replay window exceeded: diff = {}s",
                diff
            )));
        }

        let destination = header.destination.ok_or_else(|| {
            Error::Protocol("Shadowsocks-2022 request missing target destination".into())
        })?;

        // 3. Send server response header
        let resp = SessionHeader::new_server(header.session_id);
        resp.encode(&mut stream).await?;

        let mut session = SessionContext::new(&self.tag, destination);
        session.source = Some(remote_addr);
        session.destination.network = Network::Tcp;

        Ok(InboundResult { stream, session })
    }
}
