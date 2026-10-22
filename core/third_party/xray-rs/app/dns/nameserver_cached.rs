use crate::app::dns::cache_controller::CacheController;
use crate::app::dns::nameserver::NameServer;
use crate::common::errors::Result;
use async_trait::async_trait;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

pub struct CachedNameServer {
    server: Arc<dyn NameServer>,
    cache: CacheController,
    ttl: Duration,
}

impl CachedNameServer {
    pub fn new(server: Arc<dyn NameServer>, ttl: Duration) -> Self {
        Self {
            cache: CacheController::new(
                format!("cache-{}", server.name()),
                false,
                true,
                Duration::from_secs(3600),
            ),
            server,
            ttl,
        }
    }
}

#[async_trait]
impl NameServer for CachedNameServer {
    fn name(&self) -> &str {
        self.server.name()
    }

    async fn query_ip(&self, domain: &str) -> Result<Vec<IpAddr>> {
        if let Some(ips) = self.cache.get(domain) {
            return Ok(ips);
        }

        let ips = self.server.query_ip(domain).await?;
        self.cache.set(domain, ips.clone(), self.ttl);
        Ok(ips)
    }
}
