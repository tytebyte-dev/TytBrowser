// Module: transport\internet\hysteria\udphop\conn.rs
// 1:1 Rust implementation corresponding to Go transport\internet\hysteria\udphop\conn.go

use super::addr::UDPHopAddr;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

pub const DEFAULT_HOP_INTERVAL: Duration = Duration::from_secs(30);

pub struct UdpHopPacketConn {
    pub hop_addr: UDPHopAddr,
    pub addrs: Vec<SocketAddr>,
    pub hop_interval_min: Duration,
    pub hop_interval_max: Duration,
    addr_index: Arc<AtomicUsize>,
}

impl UdpHopPacketConn {
    pub fn new(
        hop_addr: UDPHopAddr,
        interval_min: Duration,
        interval_max: Duration,
    ) -> Result<Self, &'static str> {
        let (min, max) = if interval_min.is_zero() || interval_max.is_zero() {
            (DEFAULT_HOP_INTERVAL, DEFAULT_HOP_INTERVAL)
        } else {
            (interval_min, interval_max)
        };

        if min < Duration::from_secs(5) || max < Duration::from_secs(5) {
            return Err("hop interval must be at least 5 seconds");
        }

        let addrs = hop_addr.addrs();
        if addrs.is_empty() {
            return Err("no valid hop addresses");
        }

        Ok(Self {
            hop_addr,
            addrs,
            hop_interval_min: min,
            hop_interval_max: max,
            addr_index: Arc::new(AtomicUsize::new(0)),
        })
    }

    pub fn current_addr(&self) -> SocketAddr {
        let idx = self.addr_index.load(Ordering::Relaxed) % self.addrs.len();
        self.addrs[idx]
    }

    pub fn hop(&self) -> SocketAddr {
        let prev = self.addr_index.fetch_add(1, Ordering::Relaxed);
        let next_idx = (prev + 1) % self.addrs.len();
        self.addrs[next_idx]
    }
}
