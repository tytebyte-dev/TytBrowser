// Module: app\log\command\command.rs
// 1:1 Rust implementation corresponding to Go app\log\command\command.go

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct LoggerServer {
    restarted: Arc<AtomicBool>,
}

impl LoggerServer {
    pub fn new() -> Self {
        Self {
            restarted: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn restart_logger(&self) -> bool {
        self.restarted.store(true, Ordering::SeqCst);
        true
    }

    pub fn is_restarted(&self) -> bool {
        self.restarted.load(Ordering::SeqCst)
    }
}

impl Default for LoggerServer {
    fn default() -> Self {
        Self::new()
    }
}
