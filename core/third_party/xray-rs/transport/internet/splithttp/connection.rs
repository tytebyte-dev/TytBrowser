// Module: transport\internet\splithttp\connection.rs
// 1:1 Rust implementation corresponding to Go transport\internet\splithttp\connection.go

use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub type CloseCallback = Arc<dyn Fn() + Send + Sync>;

pub struct SplitConn {
    reader: Pin<Box<dyn AsyncRead + Send + Sync>>,
    writer: Pin<Box<dyn AsyncWrite + Send + Sync>>,
    local_addr: Option<SocketAddr>,
    remote_addr: Option<SocketAddr>,
    on_close: Option<CloseCallback>,
    closed: AtomicBool,
}

impl SplitConn {
    pub fn new(
        reader: Pin<Box<dyn AsyncRead + Send + Sync>>,
        writer: Pin<Box<dyn AsyncWrite + Send + Sync>>,
        local_addr: Option<SocketAddr>,
        remote_addr: Option<SocketAddr>,
        on_close: Option<CloseCallback>,
    ) -> Self {
        Self {
            reader,
            writer,
            local_addr,
            remote_addr,
            on_close,
            closed: AtomicBool::new(false),
        }
    }

    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }

    pub fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote_addr
    }

    pub fn trigger_close(&self) {
        if !self.closed.swap(true, Ordering::SeqCst)
            && let Some(ref cb) = self.on_close
        {
            cb();
        }
    }
}

impl Drop for SplitConn {
    fn drop(&mut self) {
        self.trigger_close();
    }
}

impl AsyncRead for SplitConn {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.reader.as_mut().poll_read(cx, buf)
    }
}

impl AsyncWrite for SplitConn {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.writer.as_mut().poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.writer.as_mut().poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.trigger_close();
        self.writer.as_mut().poll_shutdown(cx)
    }
}

// Session state tracking wrapper for dialer / hub session mapping
pub struct SplitHttpConnection {
    pub session_id: String,
    pub seq_id: AtomicU64,
}

impl SplitHttpConnection {
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            seq_id: AtomicU64::new(0),
        }
    }

    pub fn next_seq(&self) -> u64 {
        self.seq_id.fetch_add(1, Ordering::SeqCst)
    }
}
