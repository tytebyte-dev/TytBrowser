// Module: app\stats\channel.rs
// 1:1 Rust implementation corresponding to Go app\stats\channel.go

use super::config_pb::ChannelConfig;
use crate::common::errors::{Error, Result};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::broadcast;

pub struct StatsChannel {
    tx: broadcast::Sender<i64>,
    subscriber_limit: usize,
    running: Arc<AtomicBool>,
}

impl StatsChannel {
    pub fn new(buffer_size: usize, subscriber_limit: usize) -> Self {
        let (tx, _) = broadcast::channel(buffer_size.max(16));
        Self {
            tx,
            subscriber_limit,
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn from_config(config: &ChannelConfig) -> Self {
        Self::new(
            config.buffer_size.max(0) as usize,
            config.subscriber_limit.max(0) as usize,
        )
    }

    pub fn subscribe(&self) -> Result<broadcast::Receiver<i64>> {
        if !self.is_running() {
            return Err(Error::Closed);
        }
        if self.subscriber_limit > 0 && self.tx.receiver_count() >= self.subscriber_limit {
            return Err(Error::Config(
                "Stats channel subscriber limit reached".into(),
            ));
        }
        Ok(self.tx.subscribe())
    }

    pub fn publish(&self, value: i64) -> Result<usize> {
        if !self.is_running() {
            return Err(Error::Closed);
        }
        self.tx.send(value).map_err(|_| Error::Closed)
    }

    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn close(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

pub type Channel = StatsChannel;
