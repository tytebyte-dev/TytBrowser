// Module: transport\internet\splithttp\mux.rs
// 1:1 Rust implementation corresponding to Go transport\internet\splithttp\mux.go

use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct XmuxClient {
    pub id: u64,
    pub open_usage: AtomicI32,
    pub left_usage: AtomicI32,
    pub left_requests: AtomicI32,
    pub unreusable_at: Option<Instant>,
    pub is_closed: bool,
}

impl XmuxClient {
    pub fn new(id: u64, max_reuse: i32, max_requests: i32, max_secs: u64) -> Self {
        let unreusable_at = if max_secs > 0 {
            Some(Instant::now() + Duration::from_secs(max_secs))
        } else {
            None
        };

        Self {
            id,
            open_usage: AtomicI32::new(0),
            left_usage: AtomicI32::new(max_reuse),
            left_requests: AtomicI32::new(max_requests),
            unreusable_at,
            is_closed: false,
        }
    }

    pub fn is_reusable(&self) -> bool {
        if self.is_closed {
            return false;
        }
        if self.left_usage.load(Ordering::SeqCst) == 0 {
            return false;
        }
        if self.left_requests.load(Ordering::SeqCst) <= 0 {
            return false;
        }
        if let Some(exp) = self.unreusable_at
            && Instant::now() >= exp
        {
            return false;
        }
        true
    }

    pub fn acquire(&self) -> bool {
        if !self.is_reusable() {
            return false;
        }
        self.open_usage.fetch_add(1, Ordering::SeqCst);
        let left = self.left_usage.load(Ordering::SeqCst);
        if left > 0 {
            self.left_usage.fetch_sub(1, Ordering::SeqCst);
        }
        self.left_requests.fetch_sub(1, Ordering::SeqCst);
        true
    }

    pub fn release(&self) {
        self.open_usage.fetch_sub(1, Ordering::SeqCst);
    }
}

pub struct XmuxManager {
    pub max_concurrency: i32,
    pub max_connections: usize,
    pub max_reuse: i32,
    pub max_requests: i32,
    pub max_secs: u64,
    pub clients: Vec<Arc<Mutex<XmuxClient>>>,
    pub next_id: u64,
}

impl XmuxManager {
    pub fn new(
        max_concurrency: i32,
        max_connections: usize,
        max_reuse: i32,
        max_requests: i32,
        max_secs: u64,
    ) -> Self {
        Self {
            max_concurrency: max_concurrency.max(1),
            max_connections: max_connections.max(1),
            max_reuse,
            max_requests,
            max_secs,
            clients: Vec::new(),
            next_id: 1,
        }
    }

    pub async fn get_client(&mut self) -> Arc<Mutex<XmuxClient>> {
        // Prune expired / dead clients
        self.clients.retain(|c| {
            if let Ok(guard) = c.try_lock() {
                guard.is_reusable()
            } else {
                true
            }
        });

        // Try existing eligible client
        for c in &self.clients {
            let guard = c.lock().await;
            if guard.open_usage.load(Ordering::SeqCst) < self.max_concurrency && guard.is_reusable()
            {
                drop(guard);
                return c.clone();
            }
        }

        // Create new client if under max_connections or all busy
        let id = self.next_id;
        self.next_id += 1;
        let client = Arc::new(Mutex::new(XmuxClient::new(
            id,
            self.max_reuse,
            self.max_requests,
            self.max_secs,
        )));
        self.clients.push(client.clone());
        client
    }

    pub fn default_mux() -> Self {
        Self::new(8, 4, 100, 1000, 300)
    }

    pub async fn register(&mut self, _session_id: String) {
        let _ = self.get_client().await;
    }
}

pub type SplitHttpMux = XmuxManager;
