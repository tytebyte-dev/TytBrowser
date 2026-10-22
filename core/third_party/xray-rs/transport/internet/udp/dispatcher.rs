// Module: transport\internet\udp\dispatcher.rs
// 1:1 Rust implementation corresponding to Go transport\internet\udp\dispatcher.go

use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{Mutex, RwLock, mpsc, oneshot};

use crate::common::buf::Buffer;
use crate::common::errors::{Error, Result};
use crate::common::net::Destination;
use crate::common::protocol::udp::UdpPacket;
use crate::transport::link::Link;

pub type ResponseCallback = Arc<dyn Fn(&UdpPacket) + Send + Sync>;

#[async_trait]
pub trait LinkDispatcher: Send + Sync {
    async fn dispatch(&self, dest: Destination) -> Result<Link>;
}

#[async_trait]
impl<F> LinkDispatcher for F
where
    F: Fn(Destination) -> Result<Link> + Send + Sync,
{
    async fn dispatch(&self, dest: Destination) -> Result<Link> {
        (self)(dest)
    }
}

pub struct ConnEntry {
    pub link: Link,
    pub cancel_tx: Mutex<Option<oneshot::Sender<()>>>,
    pub closed: Arc<AtomicBool>,
}

impl ConnEntry {
    pub fn close(&self) {
        if !self.closed.swap(true, Ordering::SeqCst) {
            if let Ok(mut guard) = self.cancel_tx.try_lock()
                && let Some(tx) = guard.take()
            {
                let _ = tx.send(());
            }
            let link = self.link.clone();
            tokio::spawn(async move {
                link.reader.interrupt().await;
                link.writer.interrupt().await;
            });
        }
    }
}

pub struct Dispatcher {
    conn: Arc<RwLock<Option<Arc<ConnEntry>>>>,
    dispatcher: Arc<dyn LinkDispatcher>,
    callback: ResponseCallback,
    call_close: Option<Arc<dyn Fn() + Send + Sync>>,
    closed: Arc<AtomicBool>,
}

pub type UdpDispatcher = Dispatcher;

impl Dispatcher {
    pub fn new(dispatcher: Arc<dyn LinkDispatcher>, callback: ResponseCallback) -> Self {
        Self {
            conn: Arc::new(RwLock::new(None)),
            dispatcher,
            callback,
            call_close: None,
            closed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_call_close<F: Fn() + Send + Sync + 'static>(mut self, call_close: F) -> Self {
        self.call_close = Some(Arc::new(call_close));
        self
    }

    pub async fn remove_ray(&self) {
        self.closed.store(true, Ordering::SeqCst);
        let mut guard = self.conn.write().await;
        if let Some(entry) = guard.take() {
            entry.close();
        }
    }

    pub async fn get_inbound_ray(&self, dest: Destination) -> Result<Arc<ConnEntry>> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(Error::Other("dispatcher is closed".into()));
        }

        {
            let guard = self.conn.read().await;
            if let Some(entry) = &*guard
                && !entry.closed.load(Ordering::SeqCst)
            {
                return Ok(entry.clone());
            }
        }

        let mut guard = self.conn.write().await;
        if self.closed.load(Ordering::SeqCst) {
            return Err(Error::Other("dispatcher is closed".into()));
        }
        if let Some(entry) = &*guard
            && !entry.closed.load(Ordering::SeqCst)
        {
            return Ok(entry.clone());
        }

        let link = self.dispatcher.dispatch(dest.clone()).await?;
        let closed = Arc::new(AtomicBool::new(false));
        let (cancel_tx, mut cancel_rx) = oneshot::channel();
        let entry = Arc::new(ConnEntry {
            link: link.clone(),
            cancel_tx: Mutex::new(Some(cancel_tx)),
            closed: closed.clone(),
        });
        *guard = Some(entry.clone());

        let callback = self.callback.clone();
        let call_close = self.call_close.clone();
        let entry_clone = entry.clone();
        let dest_clone = dest.clone();

        tokio::spawn(async move {
            let reader = link.reader;
            loop {
                tokio::select! {
                    _ = &mut cancel_rx => {
                        break;
                    }
                    res = reader.read_multi_buffer() => {
                        match res {
                            Ok(mb) => {
                                if mb.is_empty() {
                                    break;
                                }
                                for buf in mb.into_buffers() {
                                    let packet = UdpPacket::new(buf, dest_clone.clone(), Destination::default());
                                    callback(&packet);
                                }
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }
                }
            }
            entry_clone.close();
            if let Some(close_fn) = call_close {
                close_fn();
            }
        });

        Ok(entry)
    }

    pub async fn dispatch(&self, destination: Destination, payload: Buffer) -> Result<()> {
        let entry = self.get_inbound_ray(destination).await?;
        entry.link.writer.write_buffer(payload).await
    }
}

pub struct DispatcherConn {
    dispatcher: Arc<Dispatcher>,
    cache_rx: Mutex<mpsc::Receiver<UdpPacket>>,
    closed: Arc<AtomicBool>,
}

impl DispatcherConn {
    pub async fn read_from(&self, buf: &mut [u8]) -> Result<(usize, Destination)> {
        let mut guard = self.cache_rx.lock().await;
        match guard.recv().await {
            Some(packet) => {
                let bytes = packet.payload.as_slice();
                let n = bytes.len().min(buf.len());
                buf[..n].copy_from_slice(&bytes[..n]);
                Ok((n, packet.source))
            }
            None => Err(Error::Other("dispatcher conn closed".into())),
        }
    }

    pub async fn write_to(&self, buf: &[u8], dest: Destination) -> Result<usize> {
        let buffer = Buffer::from_bytes(buf);
        self.dispatcher.dispatch(dest, buffer).await?;
        Ok(buf.len())
    }

    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    pub async fn close(&self) {
        if !self.closed.swap(true, Ordering::SeqCst) {
            self.dispatcher.remove_ray().await;
        }
    }
}

pub async fn dial_dispatcher(dispatcher: Arc<dyn LinkDispatcher>) -> Result<DispatcherConn> {
    let (cache_tx, cache_rx) = mpsc::channel(16);
    let closed = Arc::new(AtomicBool::new(false));
    let closed_clone = closed.clone();

    let callback: ResponseCallback = Arc::new(move |packet: &UdpPacket| {
        if !closed_clone.load(Ordering::Relaxed) {
            let _ = cache_tx.try_send(packet.clone());
        }
    });

    let d = Arc::new(Dispatcher::new(dispatcher, callback));
    Ok(DispatcherConn {
        dispatcher: d,
        cache_rx: Mutex::new(cache_rx),
        closed,
    })
}
