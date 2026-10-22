// Module: common\peer\peer.rs
// 1:1 Rust implementation corresponding to Go common\peer\peer.go

use super::latency::{AverageLatency, HasLatency, Latency};
use std::net::SocketAddr;

#[derive(Debug, Default)]
pub struct Peer {
    pub addr: Option<SocketAddr>,
    pub conn_latency: AverageLatency,
    pub handshake_latency: AverageLatency,
}

impl Peer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_addr(addr: SocketAddr) -> Self {
        Self {
            addr: Some(addr),
            conn_latency: AverageLatency::default(),
            handshake_latency: AverageLatency::default(),
        }
    }
}

impl HasLatency for Peer {
    fn connection_latency(&self) -> &dyn Latency {
        &self.conn_latency
    }

    fn handshake_latency(&self) -> &dyn Latency {
        &self.handshake_latency
    }
}
