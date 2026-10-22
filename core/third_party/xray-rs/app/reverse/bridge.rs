// Module: app\reverse\bridge.rs
// 1:1 Rust implementation corresponding to Go app\reverse\bridge.go

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::Mutex;

pub struct BridgeWorker {
    pub id: u32,
    pub active: bool,
    pub connections: AtomicU32,
}

impl BridgeWorker {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            active: true,
            connections: AtomicU32::new(0),
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn connections(&self) -> u32 {
        self.connections.load(Ordering::SeqCst)
    }

    pub fn inc_connections(&self) {
        self.connections.fetch_add(1, Ordering::SeqCst);
    }

    pub fn dec_connections(&self) {
        self.connections.fetch_sub(1, Ordering::SeqCst);
    }
}

pub struct ReverseBridge {
    pub tag: String,
    pub domain: String,
    pub workers: Arc<Mutex<Vec<Arc<BridgeWorker>>>>,
    pub next_worker_id: AtomicU32,
}

impl ReverseBridge {
    pub fn new(tag: impl Into<String>, domain: impl Into<String>) -> Self {
        let tag = tag.into();
        let domain = domain.into();
        let worker = Arc::new(BridgeWorker::new(1));
        Self {
            tag,
            domain,
            workers: Arc::new(Mutex::new(vec![worker])),
            next_worker_id: AtomicU32::new(2),
        }
    }

    pub async fn active_worker_count(&self) -> usize {
        let guard = self.workers.lock().await;
        guard.iter().filter(|w| w.is_active()).count()
    }

    pub async fn total_connections(&self) -> u32 {
        let guard = self.workers.lock().await;
        guard.iter().map(|w| w.connections()).sum()
    }

    pub async fn spawn_worker_if_needed(&self) -> Arc<BridgeWorker> {
        let mut guard = self.workers.lock().await;
        let id = self.next_worker_id.fetch_add(1, Ordering::SeqCst);
        let worker = Arc::new(BridgeWorker::new(id));
        guard.push(worker.clone());
        worker
    }
}
