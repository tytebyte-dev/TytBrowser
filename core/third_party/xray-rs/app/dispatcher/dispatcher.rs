use crate::app::dispatcher::DefaultDispatcher;
use crate::app::dns::fakedns::FakeDnsHolder;
use crate::features::outbound::{OutboundHandler, OutboundManager};
use crate::features::policy::PolicyManager;
use crate::features::routing::RouterFeature;
use crate::features::stats::StatsManagerTrait;
use std::collections::HashMap;
use std::sync::Arc;

pub struct DispatcherBuilder {
    router: Option<Arc<dyn RouterFeature>>,
    outbounds: HashMap<String, Arc<dyn OutboundHandler>>,
    default_outbound_tag: Option<String>,
    outbound_manager: Option<Arc<dyn OutboundManager>>,
    policy: Option<Arc<dyn PolicyManager>>,
    stats: Option<Arc<dyn StatsManagerTrait>>,
    fakedns: Option<Arc<FakeDnsHolder>>,
}

impl Default for DispatcherBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DispatcherBuilder {
    pub fn new() -> Self {
        Self {
            router: None,
            outbounds: HashMap::new(),
            default_outbound_tag: None,
            outbound_manager: None,
            policy: None,
            stats: None,
            fakedns: None,
        }
    }

    pub fn with_router(mut self, router: Arc<dyn RouterFeature>) -> Self {
        self.router = Some(router);
        self
    }

    pub fn add_outbound(
        mut self,
        tag: impl Into<String>,
        outbound: Arc<dyn OutboundHandler>,
    ) -> Self {
        self.outbounds.insert(tag.into(), outbound);
        self
    }

    pub fn with_default_outbound(mut self, tag: impl Into<String>) -> Self {
        self.default_outbound_tag = Some(tag.into());
        self
    }

    pub fn with_outbound_manager(mut self, om: Arc<dyn OutboundManager>) -> Self {
        self.outbound_manager = Some(om);
        self
    }

    pub fn with_policy(mut self, policy: Arc<dyn PolicyManager>) -> Self {
        self.policy = Some(policy);
        self
    }

    pub fn with_stats(mut self, stats: Arc<dyn StatsManagerTrait>) -> Self {
        self.stats = Some(stats);
        self
    }

    pub fn with_fakedns(mut self, fakedns: Arc<FakeDnsHolder>) -> Self {
        self.fakedns = Some(fakedns);
        self
    }

    pub fn build(self) -> Option<DefaultDispatcher> {
        let router = self.router?;
        let mut d = DefaultDispatcher::new(self.outbounds, router);
        if let Some(tag) = self.default_outbound_tag {
            d = d.with_default_outbound(tag);
        }
        if let Some(om) = self.outbound_manager {
            d = d.with_outbound_manager(om);
        }
        if let Some(p) = self.policy {
            d = d.with_policy(p);
        }
        if let Some(s) = self.stats {
            d = d.with_stats(s);
        }
        if let Some(f) = self.fakedns {
            d = d.with_fakedns(f);
        }
        Some(d)
    }
}
