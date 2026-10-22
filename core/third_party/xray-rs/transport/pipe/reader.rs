// Module: transport\pipe\reader.rs
// 1:1 Rust implementation corresponding to Go transport\pipe\reader.go

use std::sync::Arc;
use std::time::Duration;

use super::impl_::Pipe;
use crate::common::buf::MultiBuffer;
use crate::common::errors::{Error, Result};

#[derive(Clone)]
pub struct Reader {
    pub(crate) pipe: Arc<Pipe>,
}

impl Reader {
    pub fn new(pipe: Arc<Pipe>) -> Self {
        Self { pipe }
    }

    pub async fn read_multi_buffer(&self) -> Result<MultiBuffer> {
        self.pipe.read_multi_buffer().await
    }

    pub async fn read_multi_buffer_timeout(&self, d: Duration) -> Result<MultiBuffer> {
        self.pipe.read_multi_buffer_timeout(d).await
    }

    pub async fn interrupt(&self) {
        self.pipe.interrupt().await;
    }

    pub async fn return_an_error(&self, err: Error) {
        let mut guard = self.pipe.inner.lock().await;
        guard.pending_error = Some(err);
        self.pipe.read_signal.notify_waiters();
    }

    pub async fn recover(&self) -> Option<Error> {
        let mut guard = self.pipe.inner.lock().await;
        guard.pending_error.take()
    }
}
