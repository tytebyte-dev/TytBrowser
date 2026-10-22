// Module: transport\internet\splithttp\upload_queue.rs
// 1:1 Rust implementation corresponding to Go transport\internet\splithttp\upload_queue.go

use crate::common::errors::{Error, Result};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Packet {
    pub payload: Vec<u8>,
    pub seq: u64,
}

impl Ord for Packet {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap
        other.seq.cmp(&self.seq)
    }
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct UploadQueueInner {
    pub heap: BinaryHeap<Packet>,
    pub next_seq: u64,
    pub closed: bool,
    pub max_packets: usize,
}

#[derive(Clone)]
pub struct UploadQueue {
    inner: Arc<Mutex<UploadQueueInner>>,
}

impl UploadQueue {
    pub fn new(max_packets: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(UploadQueueInner {
                heap: BinaryHeap::new(),
                next_seq: 0,
                closed: false,
                max_packets,
            })),
        }
    }

    pub async fn push(&self, payload: Vec<u8>, seq: u64) -> Result<()> {
        let mut inner = self.inner.lock().await;
        if inner.closed {
            return Err(Error::Closed);
        }
        if inner.heap.len() >= inner.max_packets {
            return Err(Error::Protocol("packet queue is too large".into()));
        }
        inner.heap.push(Packet { payload, seq });
        Ok(())
    }

    pub async fn push_bytes(&self, data: Vec<u8>) -> Result<()> {
        let seq = {
            let inner = self.inner.lock().await;
            inner.next_seq
        };
        self.push(data, seq).await
    }

    pub async fn pop(&self) -> Option<Vec<u8>> {
        let mut buf = vec![0u8; 65535];
        let n = self.read(&mut buf).await.ok()?;
        if n == 0 {
            None
        } else {
            buf.truncate(n);
            Some(buf)
        }
    }

    pub async fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let mut inner = self.inner.lock().await;
        let mut written = 0;

        while written < buf.len() {
            if let Some(top) = inner.heap.peek() {
                if top.seq == inner.next_seq {
                    let mut packet = inner.heap.pop().unwrap();
                    let take = (buf.len() - written).min(packet.payload.len());
                    buf[written..written + take].copy_from_slice(&packet.payload[..take]);
                    written += take;

                    if take < packet.payload.len() {
                        packet.payload.drain(..take);
                        inner.heap.push(packet);
                        break;
                    } else {
                        inner.next_seq += 1;
                    }
                } else if top.seq < inner.next_seq {
                    // Stale / duplicate packet, discard
                    inner.heap.pop();
                } else {
                    // Out-of-order packet (seq > next_seq), wait for missing seq
                    break;
                }
            } else {
                break;
            }
        }

        Ok(written)
    }

    pub async fn close(&self) {
        let mut inner = self.inner.lock().await;
        inner.closed = true;
    }

    pub async fn is_closed(&self) -> bool {
        let inner = self.inner.lock().await;
        inner.closed
    }
}
