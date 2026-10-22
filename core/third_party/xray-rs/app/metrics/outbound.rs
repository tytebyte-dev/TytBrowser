// Module: app\metrics\outbound.rs
// 1:1 Rust implementation corresponding to Go app\metrics\outbound.go

use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;

use crate::app::stats::Counter;
use crate::common::errors::{Error, Result};
use crate::common::net::BoxStream;
use crate::common::protocol::SessionContext;
use crate::features::outbound::OutboundHandler;
use crate::transport::internet::stat::StatStream;

/// OutboundListener is a listener for metrics HTTP connections.
pub struct OutboundListener {
    sender: mpsc::Sender<BoxStream>,
    receiver: tokio::sync::Mutex<mpsc::Receiver<BoxStream>>,
    closed: Arc<AtomicBool>,
}

impl OutboundListener {
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);
        Self {
            sender,
            receiver: tokio::sync::Mutex::new(receiver),
            closed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn add(&self, stream: BoxStream) -> bool {
        if self.closed.load(Ordering::Acquire) {
            return false;
        }
        self.sender.try_send(stream).is_ok()
    }

    pub async fn accept(&self) -> Result<BoxStream> {
        if self.closed.load(Ordering::Acquire) {
            return Err(Error::Closed);
        }
        let mut rx = self.receiver.lock().await;
        match rx.recv().await {
            Some(stream) => Ok(stream),
            None => Err(Error::Closed),
        }
    }

    pub fn close(&self) {
        self.closed.store(true, Ordering::Release);
    }

    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }
}

/// Outbound is an outbound handler that handles metrics HTTP connections.
pub struct Outbound {
    pub tag: String,
    pub listener: Arc<OutboundListener>,
    pub closed: Arc<AtomicBool>,
}

impl Outbound {
    pub fn new(tag: impl Into<String>, listener: Arc<OutboundListener>) -> Self {
        Self {
            tag: tag.into(),
            listener,
            closed: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait]
impl OutboundHandler for Outbound {
    fn tag(&self) -> &str {
        &self.tag
    }

    async fn start(&self) -> Result<()> {
        self.closed.store(false, Ordering::Release);
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        self.closed.store(true, Ordering::Release);
        self.listener.close();
        Ok(())
    }

    async fn connect(&self, _session: &SessionContext) -> Result<BoxStream> {
        if self.closed.load(Ordering::Acquire) {
            return Err(Error::Closed);
        }
        let (client, server) = tokio::io::duplex(4096);
        let boxed_server: BoxStream = Box::pin(server);
        if !self.listener.add(boxed_server) {
            return Err(Error::BufferOverflow);
        }
        Ok(Box::pin(client))
    }
}

/// MetricsOutbound wraps an underlying OutboundHandler with read/write traffic stat counters.
pub struct MetricsOutbound {
    inner: Arc<dyn OutboundHandler>,
    read_counter: Arc<Counter>,
    write_counter: Arc<Counter>,
}

impl MetricsOutbound {
    pub fn new(
        inner: Arc<dyn OutboundHandler>,
        read_counter: Arc<Counter>,
        write_counter: Arc<Counter>,
    ) -> Self {
        Self {
            inner,
            read_counter,
            write_counter,
        }
    }
}

#[async_trait]
impl OutboundHandler for MetricsOutbound {
    fn tag(&self) -> &str {
        self.inner.tag()
    }

    async fn connect(&self, session: &SessionContext) -> Result<BoxStream> {
        let stream = self.inner.connect(session).await?;
        let stat_stream = StatStream::new(
            stream,
            Some(self.read_counter.clone()),
            Some(self.write_counter.clone()),
        );
        Ok(Box::pin(stat_stream))
    }

    async fn start(&self) -> Result<()> {
        self.inner.start().await
    }

    async fn close(&self) -> Result<()> {
        self.inner.close().await
    }
}
