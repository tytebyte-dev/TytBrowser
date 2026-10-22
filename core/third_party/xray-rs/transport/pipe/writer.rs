// Module: transport\pipe\writer.rs
// 1:1 Rust implementation corresponding to Go transport\pipe\writer.go

use std::sync::Arc;

use super::impl_::Pipe;
use crate::common::buf::{Buffer, MultiBuffer};
use crate::common::errors::Result;

#[derive(Clone)]
pub struct Writer {
    pub(crate) pipe: Arc<Pipe>,
}

impl Writer {
    pub fn new(pipe: Arc<Pipe>) -> Self {
        Self { pipe }
    }

    pub async fn write_multi_buffer(&self, mb: MultiBuffer) -> Result<()> {
        self.pipe.write_multi_buffer(mb).await
    }

    pub async fn write_buffer(&self, buf: Buffer) -> Result<()> {
        let mut mb = MultiBuffer::new();
        mb.push(buf);
        self.write_multi_buffer(mb).await
    }

    pub async fn close(&self) -> Result<()> {
        self.pipe.close().await
    }

    pub async fn len(&self) -> usize {
        self.pipe.len().await
    }

    pub async fn interrupt(&self) {
        self.pipe.interrupt().await;
    }
}
