pub mod always;
pub mod worker;

#[cfg(test)]
pub mod inbound_test;

pub use always::AlwaysOnInboundHandler;

use crate::common::errors::{Error, Result};
use crate::features::inbound::{InboundHandler, InboundManager};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use worker::InboundWorker;

pub struct DefaultInboundManager {
    tagged_handlers: RwLock<HashMap<String, Arc<dyn InboundHandler>>>,
    untagged_handlers: RwLock<Vec<Arc<dyn InboundHandler>>>,
    running: RwLock<bool>,
}

impl DefaultInboundManager {
    pub fn new() -> Self {
        Self {
            tagged_handlers: RwLock::new(HashMap::new()),
            untagged_handlers: RwLock::new(Vec::new()),
            running: RwLock::new(false),
        }
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

impl Default for DefaultInboundManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InboundManager for DefaultInboundManager {
    async fn get_handler(&self, tag: &str) -> Result<Arc<dyn InboundHandler>> {
        let tagged = self.tagged_handlers.read().await;
        tagged
            .get(tag)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Inbound handler not found: {}", tag)))
    }

    async fn add_handler(&self, handler: Arc<dyn InboundHandler>) -> Result<()> {
        let tag = handler.tag().to_string();
        if tag.is_empty() {
            let mut untagged = self.untagged_handlers.write().await;
            untagged.push(handler);
        } else {
            let mut tagged = self.tagged_handlers.write().await;
            tagged.insert(tag, handler);
        }
        Ok(())
    }

    async fn remove_handler(&self, tag: &str) -> Result<()> {
        let mut tagged = self.tagged_handlers.write().await;
        if tagged.remove(tag).is_some() {
            Ok(())
        } else {
            Err(Error::NotFound(format!(
                "Inbound handler not found: {}",
                tag
            )))
        }
    }

    async fn list_handlers(&self) -> Vec<Arc<dyn InboundHandler>> {
        let mut list = Vec::new();
        let tagged = self.tagged_handlers.read().await;
        list.extend(tagged.values().cloned());
        let untagged = self.untagged_handlers.read().await;
        list.extend(untagged.iter().cloned());
        list
    }
}
