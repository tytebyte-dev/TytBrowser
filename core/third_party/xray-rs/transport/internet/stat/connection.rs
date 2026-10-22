// Module: transport\internet\stat\connection.rs
// 1:1 Rust implementation corresponding to Go transport\internet\stat\connection.go

use crate::app::stats::Counter;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct StatStream<S> {
    inner: S,
    read_counter: Option<Arc<Counter>>,
    write_counter: Option<Arc<Counter>>,
}

impl<S> StatStream<S> {
    pub fn new(
        inner: S,
        read_counter: Option<Arc<Counter>>,
        write_counter: Option<Arc<Counter>>,
    ) -> Self {
        Self {
            inner,
            read_counter,
            write_counter,
        }
    }

    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for StatStream<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let prev_len = buf.filled().len();
        let res = Pin::new(&mut self.inner).poll_read(cx, buf);
        if let Poll::Ready(Ok(())) = &res {
            let n = buf.filled().len() - prev_len;
            if n > 0
                && let Some(counter) = &self.read_counter
            {
                counter.add(n as i64);
            }
        }
        res
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for StatStream<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let res = Pin::new(&mut self.inner).poll_write(cx, buf);
        if let Poll::Ready(Ok(n)) = &res
            && *n > 0
            && let Some(counter) = &self.write_counter
        {
            counter.add(*n as i64);
        }
        res
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}
