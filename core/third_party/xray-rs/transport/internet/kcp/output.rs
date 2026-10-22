// Module: transport\internet\kcp\output.rs
// 1:1 Rust implementation corresponding to Go transport\internet\kcp\output.go

use crate::common::errors::Result;
use async_trait::async_trait;

#[async_trait]
pub trait SegmentWriter: Send + Sync {
    async fn write(&mut self, segment: &[u8]) -> Result<()>;
}
