// Module: proxy\vmess\inbound\inbound.rs
// 1:1 Rust implementation corresponding to Go proxy\vmess\inbound\inbound.go

use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;

use crate::common::errors::{Error, Result};
use crate::common::net::{BoxStream, Network};
use crate::common::protocol::{RequestCommand, SessionContext, User};
use crate::features::inbound::{InboundHandler, InboundResult};
use crate::proxy::vmess::encoding::RequestHeader;
use crate::proxy::vmess::validator::MemoryValidator;

pub struct Server {
    tag: String,
    validator: Arc<MemoryValidator>,
}

impl Server {
    pub fn new(tag: impl Into<String>, users: Vec<Uuid>) -> Self {
        let validator = MemoryValidator::new();
        for u in users {
            let _ = validator.add(User::new(u));
        }
        Self {
            tag: tag.into(),
            validator: Arc::new(validator),
        }
    }

    pub fn with_validator(tag: impl Into<String>, validator: Arc<MemoryValidator>) -> Self {
        Self {
            tag: tag.into(),
            validator,
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
        let req = RequestHeader::decode(&mut stream).await?;

        if self.validator.count() > 0 && self.validator.get(&req.user_id).is_none() {
            return Err(Error::AuthFailed(format!(
                "Unauthorized VMess user UUID: {}",
                req.user_id
            )));
        }

        let mut session = SessionContext::new(&self.tag, req.destination);
        session.source = Some(remote_addr);
        session.destination.network = match req.command {
            RequestCommand::Tcp => Network::Tcp,
            RequestCommand::Udp => Network::Udp,
            RequestCommand::Mux => Network::Tcp,
            RequestCommand::Rvs => Network::Tcp,
        };
        session.user = Some(User::new(req.user_id));

        Ok(InboundResult { stream, session })
    }
}

pub use Server as VMessInboundServer;
