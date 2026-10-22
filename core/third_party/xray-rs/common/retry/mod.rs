pub mod retry;

#[cfg(test)]
pub mod retry_test;

use crate::common::errors::{Error, Result};
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

pub use retry::{ERR_RETRY_FAILED, Retryer, Strategy, exponential_backoff, timed};

pub enum RetryStrategy {
    Timed {
        attempts: usize,
        delay: Duration,
    },
    ExponentialBackoff {
        attempts: usize,
        base_delay: Duration,
    },
}

impl RetryStrategy {
    pub fn timed(attempts: usize, delay: Duration) -> Self {
        Self::Timed { attempts, delay }
    }

    pub fn exponential_backoff(attempts: usize, base_delay: Duration) -> Self {
        Self::ExponentialBackoff {
            attempts,
            base_delay,
        }
    }

    pub async fn run<F, Fut, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let (attempts, is_exp, base) = match self {
            Self::Timed { attempts, delay } => (*attempts, false, *delay),
            Self::ExponentialBackoff {
                attempts,
                base_delay,
            } => (*attempts, true, *base_delay),
        };

        let mut last_err = Error::Other("No attempts made".into());
        for attempt in 0..attempts {
            match operation().await {
                Ok(val) => return Ok(val),
                Err(e) => {
                    last_err = e;
                    if attempt + 1 < attempts {
                        let delay = if is_exp { base * (1 << attempt) } else { base };
                        sleep(delay).await;
                    }
                }
            }
        }
        Err(Error::Other(format!(
            "All {} retry attempts failed: {}",
            attempts, last_err
        )))
    }
}
