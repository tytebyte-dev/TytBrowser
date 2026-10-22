// Module: testing\servers\tcp\tcp.rs
// TCP test echo server with customizable message processors

use crate::common::errors::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::watch;

pub type MsgProcessor = Arc<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync>;

pub fn echo_processor() -> MsgProcessor {
    Arc::new(|data: &[u8]| data.to_vec())
}

pub fn xor_processor(key: u8) -> MsgProcessor {
    Arc::new(move |data: &[u8]| data.iter().map(|b| b ^ key).collect())
}

pub struct Server {
    addr: SocketAddr,
    shutdown_tx: watch::Sender<bool>,
    connections_count: Arc<AtomicUsize>,
    bytes_received: Arc<AtomicUsize>,
}

impl Server {
    pub async fn start(
        listen_addr: impl Into<Option<SocketAddr>>,
        processor: Option<MsgProcessor>,
        send_first: Option<Vec<u8>>,
    ) -> Result<Self> {
        let addr = listen_addr
            .into()
            .unwrap_or_else(|| "127.0.0.1:0".parse().unwrap());
        let listener = TcpListener::bind(addr).await?;
        let local_addr = listener.local_addr()?;

        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        let connections_count = Arc::new(AtomicUsize::new(0));
        let bytes_received = Arc::new(AtomicUsize::new(0));

        let processor = processor.unwrap_or_else(echo_processor);
        let send_first = Arc::new(send_first);

        let conn_count_clone = connections_count.clone();
        let bytes_recv_clone = bytes_received.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    accept_res = listener.accept() => {
                        match accept_res {
                            Ok((mut stream, _)) => {
                                conn_count_clone.fetch_add(1, Ordering::SeqCst);
                                let processor = processor.clone();
                                let send_first = send_first.clone();
                                let bytes_recv = bytes_recv_clone.clone();

                                tokio::spawn(async move {
                                    if let Some(banner) = send_first.as_ref()
                                        && stream.write_all(banner).await.is_err() {
                                            return;
                                        }

                                    let mut buf = [0u8; 4096];
                                    loop {
                                        match stream.read(&mut buf).await {
                                            Ok(0) => break,
                                            Ok(n) => {
                                                bytes_recv.fetch_add(n, Ordering::SeqCst);
                                                let processed = processor(&buf[..n]);
                                                if stream.write_all(&processed).await.is_err() {
                                                    break;
                                                }
                                            }
                                            Err(_) => break,
                                        }
                                    }
                                });
                            }
                            Err(_) => break,
                        }
                    }
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            break;
                        }
                    }
                }
            }
        });

        Ok(Self {
            addr: local_addr,
            shutdown_tx,
            connections_count,
            bytes_received,
        })
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    pub fn connections(&self) -> usize {
        self.connections_count.load(Ordering::SeqCst)
    }

    pub fn bytes_received(&self) -> usize {
        self.bytes_received.load(Ordering::SeqCst)
    }

    pub fn close(&self) {
        let _ = self.shutdown_tx.send(true);
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.close();
    }
}
