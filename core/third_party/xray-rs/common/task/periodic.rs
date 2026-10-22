// Module: common\task\periodic.rs
// 1:1 Rust implementation corresponding to Go common\task\periodic.go

use crate::common::errors::Result;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::Notify;
use tokio::task::JoinHandle;

pub struct Periodic {
    pub interval: Duration,
    pub execute: Option<Arc<dyn Fn() -> Result<()> + Send + Sync>>,
    running: Arc<AtomicBool>,
    notify_stop: Arc<Notify>,
    handle: Option<JoinHandle<()>>,
}

impl Periodic {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            execute: None,
            running: Arc::new(AtomicBool::new(false)),
            notify_stop: Arc::new(Notify::new()),
            handle: None,
        }
    }

    pub fn with_execute<F>(interval: Duration, execute: F) -> Self
    where
        F: Fn() -> Result<()> + Send + Sync + 'static,
    {
        Self {
            interval,
            execute: Some(Arc::new(execute)),
            running: Arc::new(AtomicBool::new(false)),
            notify_stop: Arc::new(Notify::new()),
            handle: None,
        }
    }

    pub fn has_closed(&self) -> bool {
        !self.running.load(Ordering::Relaxed)
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn start_execute(&mut self) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        let execute = match &self.execute {
            Some(e) => e.clone(),
            None => return Ok(()),
        };

        if let Err(e) = execute() {
            self.running.store(false, Ordering::Relaxed);
            return Err(e);
        }

        let running = self.running.clone();
        let notify_stop = self.notify_stop.clone();
        let interval = self.interval;

        let handle = tokio::spawn(async move {
            while running.load(Ordering::Relaxed) {
                tokio::select! {
                    _ = notify_stop.notified() => {
                        break;
                    }
                    _ = tokio::time::sleep(interval) => {
                        if !running.load(Ordering::Relaxed) {
                            break;
                        }
                        if execute().is_err() {
                            running.store(false, Ordering::Relaxed);
                            break;
                        }
                    }
                }
            }
        });

        self.handle = Some(handle);
        Ok(())
    }

    pub fn start<F, Fut>(&mut self, mut task: F)
    where
        F: FnMut() -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }

        let running = self.running.clone();
        let notify_stop = self.notify_stop.clone();
        let interval = self.interval;

        let handle = tokio::spawn(async move {
            while running.load(Ordering::Relaxed) {
                tokio::select! {
                    _ = notify_stop.notified() => {
                        break;
                    }
                    _ = tokio::time::sleep(interval) => {
                        if !running.load(Ordering::Relaxed) {
                            break;
                        }
                        if let Err(e) = task().await {
                            tracing::debug!("Periodic task encountered error: {}", e);
                            running.store(false, Ordering::Relaxed);
                            break;
                        }
                    }
                }
            }
        });

        self.handle = Some(handle);
    }

    pub fn close(&mut self) {
        if self.running.swap(false, Ordering::SeqCst) {
            self.notify_stop.notify_waiters();
            if let Some(handle) = self.handle.take() {
                handle.abort();
            }
        }
    }
}

impl Drop for Periodic {
    fn drop(&mut self) {
        self.close();
    }
}
