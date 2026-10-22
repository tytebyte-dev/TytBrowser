// Module: proxy\trojan\server.rs
// 1:1 Rust implementation corresponding to Go proxy\trojan\server.go

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::common::errors::{Error, Result};
use crate::common::net::{BoxStream, Destination, Network};
use crate::common::protocol::{RequestCommand, SessionContext};
use crate::features::inbound::{InboundHandler, InboundResult};
use crate::proxy::trojan::protocol::RequestHeader;
use crate::proxy::trojan::validator::PasswordValidator;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrojanFallback {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub alpn: String,
    #[serde(default)]
    pub path: String,
    pub dest: String,
    #[serde(default)]
    pub xver: u64,
}

pub struct Server {
    tag: String,
    validator: Arc<PasswordValidator>,
    fallbacks: Vec<TrojanFallback>,
}

impl Server {
    pub fn new(tag: impl Into<String>, passwords: Vec<String>) -> Self {
        let validator = PasswordValidator::new();
        for pwd in passwords {
            validator.add_password(&pwd);
        }
        Self {
            tag: tag.into(),
            validator: Arc::new(validator),
            fallbacks: Vec::new(),
        }
    }

    pub fn with_validator(tag: impl Into<String>, validator: Arc<PasswordValidator>) -> Self {
        Self {
            tag: tag.into(),
            validator,
            fallbacks: Vec::new(),
        }
    }

    pub fn with_fallbacks(mut self, fallbacks: Vec<TrojanFallback>) -> Self {
        self.fallbacks = fallbacks;
        self
    }

    pub fn add_password(&self, password: &str) {
        self.validator.add_password(password);
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
        let req_res = RequestHeader::decode(&mut stream).await;

        let req = match req_res {
            Ok(r) => r,
            Err(e) => {
                if let Some(fb) = self.fallbacks.first() {
                    let fallback_dest = Destination::parse_str(&fb.dest, Network::Tcp)?;
                    let mut session = SessionContext::new(&self.tag, fallback_dest);
                    session.source = Some(remote_addr);
                    return Ok(InboundResult { stream, session });
                }
                return Err(e);
            }
        };

        if !self.validator.validate(&req.password_hash) {
            if let Some(fb) = self.fallbacks.first() {
                let fallback_dest = Destination::parse_str(&fb.dest, Network::Tcp)?;
                let mut session = SessionContext::new(&self.tag, fallback_dest);
                session.source = Some(remote_addr);
                return Ok(InboundResult { stream, session });
            }
            return Err(Error::AuthFailed("Invalid Trojan password hash".into()));
        }

        let mut session = SessionContext::new(&self.tag, req.destination);
        session.source = Some(remote_addr);
        session.destination.network = match req.command {
            RequestCommand::Tcp => Network::Tcp,
            RequestCommand::Udp => Network::Udp,
            RequestCommand::Mux => Network::Tcp,
            RequestCommand::Rvs => Network::Tcp,
        };

        Ok(InboundResult { stream, session })
    }
}

pub use Server as TrojanServer;
