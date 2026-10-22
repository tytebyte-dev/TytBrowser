// Module: app\reverse\portal.rs
// 1:1 Rust implementation corresponding to Go app\reverse\portal.go

use crate::common::errors::{Error, Result};
use crate::common::net::BoxStream;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

pub struct StaticMuxPicker {
    workers: std::sync::Mutex<Vec<String>>,
}

impl StaticMuxPicker {
    pub fn new() -> Result<Self> {
        Ok(Self {
            workers: std::sync::Mutex::new(Vec::new()),
        })
    }

    pub fn pick_available(&self) -> Result<String> {
        let guard = self.workers.lock().unwrap();
        if guard.is_empty() {
            return Err(Error::NotFound("no mux client worker available".into()));
        }
        Ok(guard[0].clone())
    }

    pub fn add_worker(&self, tag: impl Into<String>) {
        let mut guard = self.workers.lock().unwrap();
        guard.push(tag.into());
    }
}

pub fn new_static_mux_picker() -> Result<StaticMuxPicker> {
    StaticMuxPicker::new()
}

pub struct ReversePortal {
    pub tag: String,
    pub domain: String,
    channel_tx: mpsc::Sender<BoxStream>,
    channel_rx: Arc<Mutex<mpsc::Receiver<BoxStream>>>,
}

impl ReversePortal {
    pub fn new(tag: impl Into<String>, domain: impl Into<String>) -> Self {
        let (tx, rx) = mpsc::channel(128);
        Self {
            tag: tag.into(),
            domain: domain.into(),
            channel_tx: tx,
            channel_rx: Arc::new(Mutex::new(rx)),
        }
    }

    pub async fn dispatch(&self, stream: BoxStream) -> Result<()> {
        self.channel_tx
            .send(stream)
            .await
            .map_err(|_| Error::Closed)
    }

    pub async fn pull_stream(&self) -> Option<BoxStream> {
        let mut rx = self.channel_rx.lock().await;
        rx.recv().await
    }
}
