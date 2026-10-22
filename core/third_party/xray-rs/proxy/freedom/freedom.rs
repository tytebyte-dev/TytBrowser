// Module: proxy\freedom\freedom.rs
// 1:1 Rust implementation corresponding to Go proxy\freedom\freedom.go

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::net::lookup_host;

use crate::common::errors::Result;
use crate::common::net::{Address, BoxStream, Destination};
use crate::common::protocol::SessionContext;
use crate::features::outbound::OutboundHandler;
use crate::transport::internet::TcpDialer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DomainStrategy {
    #[default]
    AsIs,
    UseIP,
    UseIPv4,
    UseIPv6,
}

impl DomainStrategy {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "useip" | "use_ip" => DomainStrategy::UseIP,
            "useipv4" | "use_ipv4" | "useip4" => DomainStrategy::UseIPv4,
            "useipv6" | "use_ipv6" | "useip6" => DomainStrategy::UseIPv6,
            _ => DomainStrategy::AsIs,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FreedomConfig {
    #[serde(default)]
    pub domain_strategy: DomainStrategy,
    #[serde(default)]
    pub timeout: u32,
    pub destination_override: Option<Destination>,
    #[serde(default)]
    pub user_level: u32,
}

pub struct Handler {
    tag: String,
    pub config: FreedomConfig,
}

impl Handler {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            config: FreedomConfig::default(),
        }
    }

    pub fn with_config(tag: impl Into<String>, config: FreedomConfig) -> Self {
        Self {
            tag: tag.into(),
            config,
        }
    }

    pub fn with_strategy(mut self, strategy: DomainStrategy) -> Self {
        self.config.domain_strategy = strategy;
        self
    }

    pub fn with_destination_override(mut self, dest: Destination) -> Self {
        self.config.destination_override = Some(dest);
        self
    }

    async fn resolve_destination(&self, dest: &Destination) -> Result<Destination> {
        if let Address::Domain(ref domain) = dest.address {
            match self.config.domain_strategy {
                DomainStrategy::AsIs => Ok(dest.clone()),
                DomainStrategy::UseIP => {
                    let addr_str = format!("{}:{}", domain, dest.port);
                    if let Ok(mut addrs) = lookup_host(&addr_str).await
                        && let Some(sock_addr) = addrs.next()
                    {
                        return Ok(Destination::new(Address::from(sock_addr.ip()), dest.port));
                    }
                    Ok(dest.clone())
                }
                DomainStrategy::UseIPv4 => {
                    let addr_str = format!("{}:{}", domain, dest.port);
                    if let Ok(addrs) = lookup_host(&addr_str).await {
                        for sa in addrs {
                            if sa.is_ipv4() {
                                return Ok(Destination::new(Address::from(sa.ip()), dest.port));
                            }
                        }
                    }
                    Ok(dest.clone())
                }
                DomainStrategy::UseIPv6 => {
                    let addr_str = format!("{}:{}", domain, dest.port);
                    if let Ok(addrs) = lookup_host(&addr_str).await {
                        for sa in addrs {
                            if sa.is_ipv6() {
                                return Ok(Destination::new(Address::from(sa.ip()), dest.port));
                            }
                        }
                    }
                    Ok(dest.clone())
                }
            }
        } else {
            Ok(dest.clone())
        }
    }
}

#[async_trait]
impl OutboundHandler for Handler {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn connect(&self, session: &SessionContext) -> Result<BoxStream> {
        let dest = self
            .config
            .destination_override
            .as_ref()
            .unwrap_or(&session.destination);

        let dial_dest = self.resolve_destination(dest).await?;
        if dial_dest.network == crate::common::net::Network::Udp {
            crate::transport::internet::dialer::dial_system(&dial_dest, None).await
        } else {
            TcpDialer::dial(&dial_dest).await
        }
    }
}

pub use Handler as Client;
pub use Handler as FreedomClient;
