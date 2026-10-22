use crate::app::stats::Counter;
use crate::common::errors::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct InboundWorker {
    pub tag: String,
    pub listen: String,
    pub port: u16,
    pub running: AtomicBool,
    pub uplink_counter: Arc<Counter>,
    pub downlink_counter: Arc<Counter>,
}

impl InboundWorker {
    pub fn new(tag: impl Into<String>, listen: impl Into<String>, port: u16) -> Self {
        Self {
            tag: tag.into(),
            listen: listen.into(),
            port,
            running: AtomicBool::new(false),
            uplink_counter: Arc::new(Counter::new()),
            downlink_counter: Arc::new(Counter::new()),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn start(&self) -> Result<()> {
        self.running.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn close(&self) -> Result<()> {
        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }
}
