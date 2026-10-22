// Module: common\retry\retry.rs
// 1:1 Rust implementation corresponding to Go common\retry\retry.go

use crate::common::errors::{Error, Result};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub const ERR_RETRY_FAILED: &str = "all retry attempts failed";

pub trait Strategy: Send + Sync {
    fn on<F: FnMut() -> Result<()>>(&self, method: F) -> Result<()>;
}

#[derive(Clone)]
pub struct Retryer {
    total_attempt: usize,
    next_delay_fn: Arc<dyn Fn(usize) -> u32 + Send + Sync>,
}

impl Retryer {
    pub fn new<D>(total_attempt: usize, delay_fn: D) -> Self
    where
        D: Fn(usize) -> u32 + Send + Sync + 'static,
    {
        Self {
            total_attempt,
            next_delay_fn: Arc::new(delay_fn),
        }
    }

    /// Synchronous retry loop corresponding to Go `Strategy.On`.
    pub fn on<F: FnMut() -> Result<()>>(&self, mut method: F) -> Result<()> {
        let mut attempt = 0;
        let mut accumulated_errors = Vec::new();
        while attempt < self.total_attempt {
            match method() {
                Ok(()) => return Ok(()),
                Err(err) => {
                    accumulated_errors.push(err.to_string());
                    let delay = (self.next_delay_fn)(attempt);
                    if delay > 0 && attempt + 1 < self.total_attempt {
                        std::thread::sleep(Duration::from_millis(delay as u64));
                    }
                    attempt += 1;
                }
            }
        }
        Err(Error::Other(format!(
            "{}: {}",
            ERR_RETRY_FAILED,
            accumulated_errors.join(" > ")
        )))
    }

    /// Asynchronous retry loop for async operations.
    pub async fn on_async<F, Fut, T>(&self, mut method: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut attempt = 0;
        let mut accumulated_errors = Vec::new();
        while attempt < self.total_attempt {
            match method().await {
                Ok(val) => return Ok(val),
                Err(err) => {
                    accumulated_errors.push(err.to_string());
                    let delay = (self.next_delay_fn)(attempt);
                    if delay > 0 && attempt + 1 < self.total_attempt {
                        sleep(Duration::from_millis(delay as u64)).await;
                    }
                    attempt += 1;
                }
            }
        }
        Err(Error::Other(format!(
            "{}: {}",
            ERR_RETRY_FAILED,
            accumulated_errors.join(" > ")
        )))
    }
}

impl Strategy for Retryer {
    fn on<F: FnMut() -> Result<()>>(&self, method: F) -> Result<()> {
        self.on(method)
    }
}

/// Timed returns a retry strategy with fixed interval.
pub fn timed(attempts: usize, delay_ms: u32) -> Retryer {
    Retryer::new(attempts, move |_| delay_ms)
}

/// ExponentialBackoff returns a retry strategy with step-wise increasing delay.
pub fn exponential_backoff(attempts: usize, delay_ms: u32) -> Retryer {
    Retryer::new(attempts, move |attempt| (attempt as u32) * delay_ms)
}
