// Module: common\singbridge\pipe.rs
// 1:1 Rust implementation corresponding to Go common\singbridge\pipe.go

use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::time::timeout;

use super::error::return_error;
use crate::common::buf::{Buffer, CopyOptions, MultiBuffer, copy_stream};
use crate::common::errors::{Error, Result};
use crate::common::net::BoxStream;
use crate::transport::Link;
use crate::transport::pipe::{Reader as PipeReader, Writer as PipeWriter};

pub const READ_TIMEOUT: Duration = Duration::from_secs(300);

/// PipeConnWrapper adapts Link Reader and Writer to stream I/O with timeouts.
/// 1:1 corresponding to PipeConnWrapper in pipe.go.
pub struct PipeConnWrapper {
    reader: PipeReader,
    writer: PipeWriter,
}

impl PipeConnWrapper {
    pub fn new(reader: PipeReader, writer: PipeWriter) -> Self {
        Self { reader, writer }
    }

    pub fn from_link(link: &Link) -> Self {
        Self {
            reader: link.reader.clone(),
            writer: link.writer.clone(),
        }
    }

    pub async fn read(&mut self) -> Result<Vec<u8>> {
        match self.reader.read_multi_buffer_timeout(READ_TIMEOUT).await {
            Ok(mb) => Ok(mb.to_vec()),
            Err(e) => {
                self.reader.interrupt().await;
                Err(e)
            }
        }
    }

    pub async fn write(&self, data: &[u8]) -> Result<()> {
        let mut mb = MultiBuffer::new();
        let mut remaining = data;
        while !remaining.is_empty() {
            let chunk_size = std::cmp::min(remaining.len(), 8192);
            let mut buf = Buffer::with_capacity(chunk_size);
            let _ = buf.write(&remaining[..chunk_size]);
            mb.push(buf);
            remaining = &remaining[chunk_size..];
        }
        self.writer.write_multi_buffer(mb).await
    }

    pub async fn close(&self) -> Result<()> {
        self.writer.close().await
    }
}

/// CopyConn copies data bidirectionally between link and a remote server stream.
/// 1:1 corresponding to CopyConn() in pipe.go.
pub async fn copy_conn(link: Link, server_stream: BoxStream) -> Result<()> {
    let (reader, writer) = link.into_parts();
    let (mut server_read, mut server_write) = tokio::io::split(server_stream);

    // Task 1: link.reader -> server_stream
    let r_task = async {
        loop {
            match timeout(READ_TIMEOUT, reader.read_multi_buffer()).await {
                Ok(Ok(mb)) => {
                    let bytes = mb.to_vec();
                    if bytes.is_empty() {
                        break;
                    }
                    if let Err(e) = server_write.write_all(&bytes).await {
                        return Err(Error::Io(e));
                    }
                    if let Err(e) = server_write.flush().await {
                        return Err(Error::Io(e));
                    }
                }
                Ok(Err(e)) => return Err(e),
                Err(_) => {
                    reader.interrupt().await;
                    return Err(Error::Io(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "singbridge pipe read timeout",
                    )));
                }
            }
        }
        let _ = server_write.shutdown().await;
        Ok::<(), Error>(())
    };

    // Task 2: server_stream -> link.writer
    let w_task = async {
        let mut buf = [0u8; 8192];
        loop {
            let n = match server_read.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => return Err(Error::Io(e)),
            };

            let mut mb = MultiBuffer::new();
            let mut b = Buffer::with_capacity(n);
            let _ = b.write(&buf[..n]);
            mb.push(b);

            writer.write_multi_buffer(mb).await?;
        }
        let _ = writer.close().await;
        Ok::<(), Error>(())
    };

    let result = tokio::select! {
        r = r_task => r,
        w = w_task => w,
    };

    match result {
        Ok(_) => Ok(()),
        Err(e) => match return_error(Some(e)) {
            None => Ok(()),
            Some(err) => Err(err),
        },
    }
}

/// Backwards compatibility helper
pub async fn bridge_pipe<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    mut reader: R,
    mut writer: W,
) -> Result<u64> {
    copy_stream(&mut reader, &mut writer, 4096, CopyOptions::default()).await
}
