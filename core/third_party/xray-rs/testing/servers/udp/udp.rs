// Module: testing\servers\udp\udp.rs
// UDP test echo server

use crate::common::errors::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::net::UdpSocket;
use tokio::sync::watch;

pub struct Server {
    addr: SocketAddr,
    shutdown_tx: watch::Sender<bool>,
    packets_count: Arc<AtomicUsize>,
}

impl Server {
    pub async fn start(listen_addr: impl Into<Option<SocketAddr>>) -> Result<Self> {
        let addr = listen_addr
            .into()
            .unwrap_or_else(|| "127.0.0.1:0".parse().unwrap());
        let socket = UdpSocket::bind(addr).await?;
        let local_addr = socket.local_addr()?;

        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        let packets_count = Arc::new(AtomicUsize::new(0));
        let count_clone = packets_count.clone();

        tokio::spawn(async move {
            let mut buf = [0u8; 65535];
            loop {
                tokio::select! {
                    res = socket.recv_from(&mut buf) => {
                        match res {
                            Ok((len, peer)) => {
                                count_clone.fetch_add(1, Ordering::SeqCst);
                                let _ = socket.send_to(&buf[..len], peer).await;
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
            packets_count,
        })
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    pub fn packets(&self) -> usize {
        self.packets_count.load(Ordering::SeqCst)
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
