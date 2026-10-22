// Module: common\signal\done\done.rs
// 1:1 Rust implementation corresponding to Go common\signal\done\done.go

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Notify;

#[derive(Clone)]
pub struct Instance {
    closed: Arc<AtomicBool>,
    notify: Arc<Notify>,
}

impl Instance {
    pub fn new() -> Self {
        Self {
            closed: Arc::new(AtomicBool::new(false)),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn done(&self) -> bool {
        self.closed.load(Ordering::Relaxed)
    }

    pub fn is_done(&self) -> bool {
        self.done()
    }

    pub async fn wait(&self) {
        if self.done() {
            return;
        }
        self.notify.notified().await;
    }

    pub fn close(&self) {
        if !self.closed.swap(true, Ordering::SeqCst) {
            self.notify.notify_waiters();
        }
    }
}

impl Default for Instance {
    fn default() -> Self {
        Self::new()
    }
}

pub type Done = Instance;
