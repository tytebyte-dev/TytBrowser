pub mod handler;
pub mod uot;

#[cfg(test)]
pub mod handler_test;

use crate::common::errors::{Error, Result};
use crate::features::outbound::{HandlerSelector, OutboundHandler, OutboundManager};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use handler::DefaultOutboundHandler;
pub use uot::{
    UOT_LEGACY_MAGIC_ADDRESS, UOT_MAGIC_ADDRESS, UotPacket, UotVersion, is_uot_destination,
};

pub struct DefaultOutboundManager {
    tagged_handlers: RwLock<HashMap<String, Arc<dyn OutboundHandler>>>,
    tagged_order: RwLock<Vec<String>>,
    untagged_handlers: RwLock<Vec<Arc<dyn OutboundHandler>>>,
    default_handler: RwLock<Option<Arc<dyn OutboundHandler>>>,
    running: RwLock<bool>,
}

impl DefaultOutboundManager {
    pub fn new() -> Self {
        Self {
            tagged_handlers: RwLock::new(HashMap::new()),
            tagged_order: RwLock::new(Vec::new()),
            untagged_handlers: RwLock::new(Vec::new()),
            default_handler: RwLock::new(None),
            running: RwLock::new(false),
        }
    }

    pub async fn select_handlers(
        &self,
        selector: &dyn HandlerSelector,
    ) -> Vec<Arc<dyn OutboundHandler>> {
        let tagged = self.tagged_handlers.read().await;
        let tags: Vec<String> = tagged.keys().cloned().collect();
        let selected_tags = selector.select(&tags);

        let mut result = Vec::new();
        for tag in selected_tags {
            if let Some(h) = tagged.get(&tag) {
                result.push(h.clone());
            }
        }
        result
    }

    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = true;

        let tagged = self.tagged_handlers.read().await;
        for handler in tagged.values() {
            handler.start().await?;
        }

        let untagged = self.untagged_handlers.read().await;
        for handler in untagged.iter() {
            handler.start().await?;
        }

        Ok(())
    }

    pub async fn close(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;

        let tagged = self.tagged_handlers.read().await;
        for handler in tagged.values() {
            handler.close().await?;
        }

        let untagged = self.untagged_handlers.read().await;
        for handler in untagged.iter() {
            handler.close().await?;
        }

        Ok(())
    }
}

impl Default for DefaultOutboundManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OutboundManager for DefaultOutboundManager {
    async fn get_handler(&self, tag: &str) -> Option<Arc<dyn OutboundHandler>> {
        let tagged = self.tagged_handlers.read().await;
        tagged.get(tag).cloned()
    }

    async fn get_default_handler(&self) -> Option<Arc<dyn OutboundHandler>> {
        let def = self.default_handler.read().await;
        if def.is_some() {
            return def.clone();
        }

        let untagged = self.untagged_handlers.read().await;
        if let Some(first) = untagged.first() {
            return Some(first.clone());
        }

        let order = self.tagged_order.read().await;
        let tagged = self.tagged_handlers.read().await;
        for tag in order.iter() {
            if let Some(h) = tagged.get(tag) {
                return Some(h.clone());
            }
        }

        tagged.values().next().cloned()
    }

    async fn add_handler(&self, handler: Arc<dyn OutboundHandler>) -> Result<()> {
        let tag = handler.tag().to_string();
        if tag.is_empty() {
            let mut untagged = self.untagged_handlers.write().await;
            untagged.push(handler);
        } else {
            let mut order = self.tagged_order.write().await;
            if !order.contains(&tag) {
                order.push(tag.clone());
            }
            let mut tagged = self.tagged_handlers.write().await;
            tagged.insert(tag, handler);
        }
        Ok(())
    }

    async fn remove_handler(&self, tag: &str) -> Result<()> {
        let mut order = self.tagged_order.write().await;
        order.retain(|t| t != tag);
        let mut tagged = self.tagged_handlers.write().await;
        if tagged.remove(tag).is_some() {
            Ok(())
        } else {
            Err(Error::NotFound(format!(
                "Outbound handler not found: {}",
                tag
            )))
        }
    }

    async fn list_handlers(&self) -> Vec<Arc<dyn OutboundHandler>> {
        let mut list = Vec::new();
        let order = self.tagged_order.read().await;
        let tagged = self.tagged_handlers.read().await;
        for tag in order.iter() {
            if let Some(h) = tagged.get(tag) {
                list.push(h.clone());
            }
        }
        let untagged = self.untagged_handlers.read().await;
        list.extend(untagged.iter().cloned());
        list
    }
}
