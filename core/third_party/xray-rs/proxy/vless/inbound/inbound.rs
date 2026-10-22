// Module: proxy\vless\inbound\inbound.rs
// 1:1 Rust implementation corresponding to Go proxy\vless\inbound\inbound.go

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;

use crate::common::errors::{Error, Result};
use crate::common::net::{BoxStream, Destination, Network};
use crate::common::protocol::{RequestCommand, SessionContext, User};
use crate::features::inbound::{InboundHandler, InboundResult};
use crate::features::policy::PolicyManager;
use crate::features::stats::StatsManagerTrait;
use crate::proxy::vless::encoding::{RequestHeader, ResponseHeader};
use crate::proxy::vless::flow::FLOW_VISION;
use crate::proxy::vless::validator::{MemoryValidator, Validator};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fallback {
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

pub struct Handler {
    tag: String,
    validator: Arc<dyn Validator>,
    fallbacks: Vec<Fallback>,
    policy_manager: Option<Arc<dyn PolicyManager>>,
    stats_manager: Option<Arc<dyn StatsManagerTrait>>,
    fallback_map: HashMap<String, HashMap<String, HashMap<String, Fallback>>>,
}

impl Handler {
    pub fn new(tag: impl Into<String>, users: Vec<Uuid>) -> Self {
        let validator = MemoryValidator::new();
        for u in users {
            let _ = validator.add(User::new(u));
        }
        Self::with_validator(tag, Arc::new(validator))
    }

    pub fn with_validator(tag: impl Into<String>, validator: Arc<dyn Validator>) -> Self {
        Self {
            tag: tag.into(),
            validator,
            fallbacks: Vec::new(),
            policy_manager: None,
            stats_manager: None,
            fallback_map: HashMap::new(),
        }
    }

    pub fn with_fallbacks(mut self, fallbacks: Vec<Fallback>) -> Self {
        self.fallbacks = fallbacks.clone();
        let mut map: HashMap<String, HashMap<String, HashMap<String, Fallback>>> = HashMap::new();
        for fb in fallbacks {
            map.entry(fb.name.to_lowercase())
                .or_default()
                .entry(fb.alpn.to_lowercase())
                .or_default()
                .insert(fb.path.clone(), fb);
        }
        self.fallback_map = map;
        self
    }

    pub fn with_policy_manager(mut self, policy_manager: Arc<dyn PolicyManager>) -> Self {
        self.policy_manager = Some(policy_manager);
        self
    }

    pub fn with_stats_manager(mut self, stats_manager: Arc<dyn StatsManagerTrait>) -> Self {
        self.stats_manager = Some(stats_manager);
        self
    }

    pub fn add_user(&self, user: User) -> Result<()> {
        self.validator.add(user)
    }

    pub fn del_user(&self, email: &str) -> Result<()> {
        self.validator.del(email)
    }

    pub fn get_user(&self, uuid: &Uuid) -> Option<User> {
        self.validator.get(uuid)
    }

    pub fn get_user_count(&self) -> usize {
        self.validator.get_count()
    }

    pub fn match_fallback(&self, name: &str, alpn: &str, path: &str) -> Option<&Fallback> {
        let name_lower = name.to_lowercase();
        let alpn_lower = alpn.to_lowercase();

        // 1. Try exact name match
        if let Some(by_name) = self
            .fallback_map
            .get(&name_lower)
            .or_else(|| self.fallback_map.get(""))
        {
            // 2. Try ALPN match
            if let Some(by_alpn) = by_name.get(&alpn_lower).or_else(|| by_name.get("")) {
                // 3. Try path match
                if let Some(fb) = by_alpn.get(path).or_else(|| by_alpn.get("")) {
                    return Some(fb);
                }
            }
        }
        None
    }
}

#[async_trait]
impl InboundHandler for Handler {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn handle_connection(
        &self,
        mut stream: BoxStream,
        remote_addr: SocketAddr,
    ) -> Result<InboundResult> {
        // 1. Decode VLESS request header
        let req_result = RequestHeader::decode(&mut stream).await;

        let req = match req_result {
            Ok(r) => r,
            Err(e) => {
                // If fallback is configured, check if we can route to fallback
                if let Some(fb) = self.match_fallback("", "", "") {
                    let fallback_dest = Destination::parse_str(&fb.dest, Network::Tcp)?;
                    let mut session = SessionContext::new(&self.tag, fallback_dest);
                    session.source = Some(remote_addr);
                    return Ok(InboundResult { stream, session });
                }
                return Err(e);
            }
        };

        // 2. Validate User UUID
        let user = if self.validator.get_count() > 0 {
            match self.validator.get(&req.user_id) {
                Some(u) => Some(u),
                None => {
                    if let Some(fb) = self.match_fallback("", "", "") {
                        let fallback_dest = Destination::parse_str(&fb.dest, Network::Tcp)?;
                        let mut session = SessionContext::new(&self.tag, fallback_dest);
                        session.source = Some(remote_addr);
                        return Ok(InboundResult { stream, session });
                    }
                    return Err(Error::AuthFailed(format!(
                        "Unauthorized VLESS user UUID: {}",
                        req.user_id
                    )));
                }
            }
        } else {
            None
        };

        // 3. Respond with VLESS Response Header
        let resp = ResponseHeader::new();
        resp.encode(&mut stream).await?;

        // 4. Build session context
        let mut session = SessionContext::new(&self.tag, req.destination);
        session.source = Some(remote_addr);
        session.destination.network = match req.command {
            RequestCommand::Tcp => Network::Tcp,
            RequestCommand::Udp => Network::Udp,
            RequestCommand::Mux => Network::Tcp,
            RequestCommand::Rvs => Network::Tcp,
        };

        if let Some(u) = user {
            session.user = Some(u);
        } else {
            session.user = Some(User::new(req.user_id));
        }

        // 5. Flow inspection (XTLS-Vision)
        if req.addons.flow == FLOW_VISION {
            session.sniffed_protocol = Some("tls".into());
        }

        Ok(InboundResult { stream, session })
    }
}

pub use Handler as Server;
pub use Handler as VlessInboundServer;
