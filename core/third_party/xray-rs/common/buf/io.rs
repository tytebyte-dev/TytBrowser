// Module: common\buf\io.rs
// 1:1 Rust implementation corresponding to Go common\buf\io.go

use async_trait::async_trait;
use std::time::Duration;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::common::buf::MultiBuffer;
use crate::common::errors::{Error, Result};

#[async_trait]
pub trait Reader: Send + Sync {
    async fn read_multi_buffer(&mut self) -> Result<MultiBuffer>;
}

#[async_trait]
pub trait TimeoutReader: Reader {
    async fn read_multi_buffer_timeout(&mut self, duration: Duration) -> Result<MultiBuffer>;
}

#[async_trait]
pub trait Writer: Send + Sync {
    async fn write_multi_buffer(&mut self, mb: MultiBuffer) -> Result<()>;
}

pub async fn write_all_bytes<W: AsyncWrite + Unpin>(
    writer: &mut W,
    payload: &[u8],
) -> Result<usize> {
    writer.write_all(payload).await.map_err(Error::Io)?;
    Ok(payload.len())
}
