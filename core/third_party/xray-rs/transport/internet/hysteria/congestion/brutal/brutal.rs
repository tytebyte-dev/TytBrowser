// Module: transport\internet\hysteria\congestion\brutal\brutal.rs
// 1:1 Rust implementation corresponding to Go transport\internet\hysteria\congestion\brutal\brutal.go

use super::super::common::pacer::Pacer;

pub const PKT_INFO_SLOT_COUNT: usize = 5;
pub const MIN_ACK_RATE: f64 = 0.8;

#[derive(Debug, Clone, Copy, Default)]
pub struct PktInfo {
    pub timestamp_sec: i64,
    pub ack_count: u64,
    pub loss_count: u64,
}

pub struct BrutalSender {
    pub bps: u64,
    pub max_datagram_size: u64,
    pub ack_rate: f64,
    pub pacer: Pacer,
    pub slots: [PktInfo; PKT_INFO_SLOT_COUNT],
}

impl BrutalSender {
    pub fn new(bps: u64) -> Self {
        Self {
            bps,
            max_datagram_size: 1200,
            ack_rate: 1.0,
            pacer: Pacer::new(bps),
            slots: [PktInfo::default(); PKT_INFO_SLOT_COUNT],
        }
    }

    pub fn pacing_rate(&self) -> u64 {
        let rate = (self.bps as f64 / self.ack_rate.max(MIN_ACK_RATE)) as u64;
        rate.max(self.bps)
    }

    pub fn record_ack(&mut self, now_sec: i64, count: u64) {
        let slot_idx = (now_sec as usize) % PKT_INFO_SLOT_COUNT;
        if self.slots[slot_idx].timestamp_sec != now_sec {
            self.slots[slot_idx] = PktInfo {
                timestamp_sec: now_sec,
                ack_count: count,
                loss_count: 0,
            };
        } else {
            self.slots[slot_idx].ack_count += count;
        }
        self.update_ack_rate(now_sec);
    }

    pub fn record_loss(&mut self, now_sec: i64, count: u64) {
        let slot_idx = (now_sec as usize) % PKT_INFO_SLOT_COUNT;
        if self.slots[slot_idx].timestamp_sec != now_sec {
            self.slots[slot_idx] = PktInfo {
                timestamp_sec: now_sec,
                ack_count: 0,
                loss_count: count,
            };
        } else {
            self.slots[slot_idx].loss_count += count;
        }
        self.update_ack_rate(now_sec);
    }

    fn update_ack_rate(&mut self, now_sec: i64) {
        let mut total_ack = 0u64;
        let mut total_loss = 0u64;
        for s in &self.slots {
            if now_sec - s.timestamp_sec < PKT_INFO_SLOT_COUNT as i64 {
                total_ack += s.ack_count;
                total_loss += s.loss_count;
            }
        }
        let total = total_ack + total_loss;
        if total > 0 {
            self.ack_rate = (total_ack as f64) / (total as f64);
        }
    }
}
