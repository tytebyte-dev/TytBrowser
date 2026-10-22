// Module: common\buf\copy.rs
// 1:1 Rust implementation corresponding to Go common\buf\copy.go

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::io::{Reader, TimeoutReader, Writer};
use crate::app::stats::Counter;
use crate::common::errors::{Error, Result};
use crate::common::signal::ActivityTimer;

/// Options for copying streams or multibuffers.
/// 1:1 corresponding to CopyOption in copy.go.
#[derive(Clone, Default)]
pub struct CopyOptions {
    pub timer: Option<Arc<ActivityTimer>>,
    pub counter: Option<Arc<Counter>>,
    pub size_counter: Option<Arc<AtomicI64>>,
}

impl CopyOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_timer(mut self, timer: Arc<ActivityTimer>) -> Self {
        self.timer = Some(timer);
        self
    }

    pub fn with_counter(mut self, counter: Arc<Counter>) -> Self {
        self.counter = Some(counter);
        self
    }

    pub fn with_size_counter(mut self, size: Arc<AtomicI64>) -> Self {
        self.size_counter = Some(size);
        self
    }
}

/// Dumps all payload from reader to writer or stops when EOF occurs.
/// 1:1 corresponding to Copy() in copy.go.
pub async fn copy<R: Reader, W: Writer>(
    reader: &mut R,
    writer: &mut W,
    options: &CopyOptions,
) -> Result<u64> {
    let mut total = 0u64;
    loop {
        match reader.read_multi_buffer().await {
            Ok(mb) => {
                if mb.is_empty() {
                    break;
                }
                let len = mb.len();
                total += len as u64;

                if let Some(timer) = &options.timer {
                    timer.update();
                }
                if let Some(counter) = &options.counter {
                    counter.add(len as i64);
                }
                if let Some(size) = &options.size_counter {
                    size.fetch_add(len as i64, Ordering::Relaxed);
                }

                writer.write_multi_buffer(mb).await?;
            }
            Err(e) => {
                if let Error::Io(ref io_err) = e
                    && io_err.kind() == std::io::ErrorKind::UnexpectedEof
                {
                    break;
                }
                return Err(e);
            }
        }
    }
    Ok(total)
}

/// Reads a multibuffer once with timeout and writes it to the writer.
/// 1:1 corresponding to CopyOnceTimeout() in copy.go.
pub async fn copy_once_timeout<R: TimeoutReader, W: Writer>(
    reader: &mut R,
    writer: &mut W,
    d: Duration,
) -> Result<usize> {
    let mb = reader.read_multi_buffer_timeout(d).await?;
    let len = mb.len();
    writer.write_multi_buffer(mb).await?;
    Ok(len)
}

/// Helper for bidirectional async byte stream copying.
pub async fn copy_stream<R, W>(
    mut reader: R,
    mut writer: W,
    buffer_size: usize,
    options: CopyOptions,
) -> Result<u64>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buf = vec![0u8; buffer_size.max(4096)];
    let mut total = 0u64;

    loop {
        let n = reader.read(&mut buf).await.map_err(Error::Io)?;
        if n == 0 {
            break;
        }

        writer.write_all(&buf[..n]).await.map_err(Error::Io)?;
        total += n as u64;

        if let Some(timer) = &options.timer {
            timer.update();
        }
        if let Some(counter) = &options.counter {
            counter.add(n as i64);
        }
        if let Some(size) = &options.size_counter {
            size.fetch_add(n as i64, Ordering::Relaxed);
        }
    }

    writer.flush().await.map_err(Error::Io)?;
    Ok(total)
}
