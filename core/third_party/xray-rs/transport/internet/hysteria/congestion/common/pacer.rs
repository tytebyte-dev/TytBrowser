// Module: transport\internet\hysteria\congestion\common\pacer.rs
// 1:1 Rust implementation corresponding to Go transport\internet\hysteria\congestion\common\pacer.go

use std::time::{Duration, Instant};

pub const MAX_BURST_PACKETS: u64 = 10;
pub const INITIAL_PACKET_SIZE: u64 = 1200;
pub const MIN_PACING_DELAY: Duration = Duration::from_micros(100);

pub struct Pacer {
    budget_at_last_sent: u64,
    max_datagram_size: u64,
    last_sent_time: Option<Instant>,
    bandwidth_bps: u64,
}

impl Pacer {
    pub fn new(bandwidth_bps: u64) -> Self {
        Self {
            budget_at_last_sent: MAX_BURST_PACKETS * INITIAL_PACKET_SIZE,
            max_datagram_size: INITIAL_PACKET_SIZE,
            last_sent_time: None,
            bandwidth_bps,
        }
    }

    pub fn set_bandwidth(&mut self, bps: u64) {
        self.bandwidth_bps = bps;
    }

    pub fn budget(&self, now: Instant) -> u64 {
        let last = match self.last_sent_time {
            Some(t) => t,
            None => return self.max_burst_size(),
        };
        let elapsed = now.duration_since(last);
        let bytes_per_sec = self.bandwidth_bps / 8;
        let accrued = (bytes_per_sec as f64 * elapsed.as_secs_f64()) as u64;
        (self.budget_at_last_sent + accrued).min(self.max_burst_size())
    }

    pub fn max_burst_size(&self) -> u64 {
        MAX_BURST_PACKETS * self.max_datagram_size
    }

    pub fn sent_packet(&mut self, send_time: Instant, size: u64) {
        let current_budget = self.budget(send_time);
        self.budget_at_last_sent = current_budget.saturating_sub(size);
        self.last_sent_time = Some(send_time);
    }

    pub fn time_until_send(&self, now: Instant) -> Duration {
        if self.budget(now) >= self.max_datagram_size {
            return Duration::ZERO;
        }
        let needed = self.max_datagram_size - self.budget_at_last_sent;
        let bytes_per_sec = (self.bandwidth_bps / 8).max(1);
        let secs = (needed as f64) / (bytes_per_sec as f64);
        Duration::from_secs_f64(secs).max(MIN_PACING_DELAY)
    }
}
