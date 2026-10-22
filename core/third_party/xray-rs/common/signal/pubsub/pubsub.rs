// Module: common\signal\pubsub\pubsub.rs
// 1:1 Rust implementation corresponding to Go common\signal\pubsub\pubsub.go

use crate::common::errors::{Error, Result};
use crate::common::signal::done::Instance as DoneInstance;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

pub struct Subscriber {
    rx: broadcast::Receiver<String>,
    done: DoneInstance,
}

impl Subscriber {
    pub fn new(rx: broadcast::Receiver<String>) -> Self {
        Self {
            rx,
            done: DoneInstance::new(),
        }
    }

    pub async fn recv(&mut self) -> Result<String> {
        self.rx
            .recv()
            .await
            .map_err(|e| Error::Protocol(e.to_string()))
    }

    pub async fn wait(&mut self) -> Option<String> {
        self.rx.recv().await.ok()
    }

    pub fn try_recv(&mut self) -> Option<String> {
        self.rx.try_recv().ok()
    }

    pub fn close(&self) -> Result<()> {
        self.done.close();
        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        self.done.done()
    }
}

#[derive(Clone)]
pub struct Service {
    topics: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>,
    capacity: usize,
}

impl Service {
    pub fn new(capacity: usize) -> Self {
        Self {
            topics: Arc::new(RwLock::new(HashMap::new())),
            capacity: capacity.max(16),
        }
    }

    pub fn new_service() -> Self {
        Self::new(16)
    }

    pub fn subscribe(&self, topic: &str) -> Subscriber {
        let mut topics = self.topics.write().unwrap();
        let sender = topics.entry(topic.to_string()).or_insert_with(|| {
            let (tx, _) = broadcast::channel(self.capacity);
            tx
        });
        Subscriber::new(sender.subscribe())
    }

    pub fn publish(&self, topic: &str, message: impl Into<String>) -> Result<usize> {
        let topics = self.topics.read().unwrap();
        if let Some(sender) = topics.get(topic) {
            sender
                .send(message.into())
                .map_err(|e| Error::Protocol(e.to_string()))
        } else {
            Ok(0)
        }
    }

    pub fn cleanup(&self) {
        let mut topics = self.topics.write().unwrap();
        topics.retain(|_, sender| sender.receiver_count() > 0);
    }
}

impl Default for Service {
    fn default() -> Self {
        Self::new(16)
    }
}

pub type PubSubService = Service;
