// Module: app\proxyman\inbound\always.rs
// 1:1 Rust implementation corresponding to Go app\proxyman\inbound\always.go

use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::worker::InboundWorker;
use crate::app::stats::{Counter, StatsManager, inbound_downlink_name, inbound_uplink_name};
use crate::common::errors::Result;
use crate::common::net::BoxStream;
use crate::common::protocol::SessionContext;
use crate::features::inbound::{InboundHandler, InboundResult};

pub const ALWAYS_ON: bool = true;

pub struct AlwaysOnInboundHandler {
    tag: String,
    workers: RwLock<Vec<InboundWorker>>,
    uplink_counter: Option<Arc<Counter>>,
    downlink_counter: Option<Arc<Counter>>,
    receiver_settings: Option<Vec<u8>>,
    proxy_settings: Option<Vec<u8>>,
}

impl AlwaysOnInboundHandler {
    pub fn new(
        tag: impl Into<String>,
        stats_mgr: Option<&StatsManager>,
        receiver_settings: Option<Vec<u8>>,
        proxy_settings: Option<Vec<u8>>,
    ) -> Self {
        let tag_str = tag.into();
        let (uplink_counter, downlink_counter) = if let Some(mgr) = stats_mgr {
            (
                Some(mgr.register_counter_sync(inbound_uplink_name(&tag_str))),
                Some(mgr.register_counter_sync(inbound_downlink_name(&tag_str))),
            )
        } else {
            (None, None)
        };

        Self {
            tag: tag_str,
            workers: RwLock::new(Vec::new()),
            uplink_counter,
            downlink_counter,
            receiver_settings,
            proxy_settings,
        }
    }

    pub async fn add_worker(&self, worker: InboundWorker) {
        self.workers.write().await.push(worker);
    }

    pub fn uplink_counter(&self) -> Option<&Arc<Counter>> {
        self.uplink_counter.as_ref()
    }

    pub fn downlink_counter(&self) -> Option<&Arc<Counter>> {
        self.downlink_counter.as_ref()
    }
}

#[async_trait]
impl InboundHandler for AlwaysOnInboundHandler {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn start(&self) -> Result<()> {
        let workers = self.workers.read().await;
        for w in workers.iter() {
            w.start()?;
        }
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        let workers = self.workers.read().await;
        for w in workers.iter() {
            w.close()?;
        }
        Ok(())
    }

    fn receiver_settings(&self) -> Option<Vec<u8>> {
        self.receiver_settings.clone()
    }

    fn proxy_settings(&self) -> Option<Vec<u8>> {
        self.proxy_settings.clone()
    }

    async fn handle_connection(
        &self,
        stream: BoxStream,
        remote_addr: SocketAddr,
    ) -> Result<InboundResult> {
        let dest = crate::common::net::Destination::from(remote_addr);
        let session = SessionContext::new(self.tag.clone(), dest);
        Ok(InboundResult { stream, session })
    }
}
