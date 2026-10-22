// Module: common\drain\drain.rs
// 1:1 Rust implementation corresponding to Go common\drain\drain.go

use crate::common::errors::Result;
use tokio::io::{AsyncRead, AsyncReadExt};

/// Drainer defines an interface for tracking and draining unconsumed connection bytes.
pub trait Drainer: Send + Sync {
    fn acknowledge_receive(&mut self, size: usize);
}

/// Helper function to drain up to `max_bytes` from an async reader.
pub async fn drain_read_n<R: AsyncRead + Unpin>(reader: &mut R, mut n: usize) -> Result<usize> {
    let mut buf = [0u8; 4096];
    let mut drained = 0;
    while n > 0 {
        let to_read = n.min(buf.len());
        let read = reader.read(&mut buf[..to_read]).await?;
        if read == 0 {
            break;
        }
        drained += read;
        n -= read;
    }
    Ok(drained)
}
