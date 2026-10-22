// Module: transport\internet\kcp\sending.rs
// 1:1 Rust implementation corresponding to Go transport\internet\kcp\sending.go

use super::segment::DataSegment;
use std::collections::VecDeque;

pub struct SendingWindow {
    pub cache: VecDeque<DataSegment>,
    pub total_in_flight_size: u32,
}

impl SendingWindow {
    pub fn new() -> Self {
        Self {
            cache: VecDeque::new(),
            total_in_flight_size: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    pub fn push(&mut self, seg: DataSegment) {
        self.total_in_flight_size += seg.payload.len() as u32;
        self.cache.push_back(seg);
    }

    pub fn first_number(&self) -> Option<u32> {
        self.cache.front().map(|s| s.number)
    }

    /// Clears all acknowledged segments with number < una
    pub fn clear(&mut self, una: u32) {
        while let Some(front) = self.cache.front() {
            // number < una
            if front.number.wrapping_sub(una) >= 0x7FFFFFFF {
                let seg = self.cache.pop_front().unwrap();
                self.total_in_flight_size = self
                    .total_in_flight_size
                    .saturating_sub(seg.payload.len() as u32);
            } else {
                break;
            }
        }
    }

    /// Fast retransmit: accelerates packet timeout when newer packet is ACKed
    pub fn handle_fast_ack(&mut self, number: u32, rto: u32) {
        for seg in &mut self.cache {
            if seg.number == number || number.wrapping_sub(seg.number) > 0x7FFFFFFF {
                break;
            }
            if seg.transmit > 0 && seg.timeout > rto / 3 {
                seg.timeout -= rto / 3;
            }
        }
    }

    /// Acknowledges a specific packet number and returns RTT if valid
    pub fn acknowledge(&mut self, number: u32, current: u32) -> Option<u32> {
        if let Some(pos) = self.cache.iter().position(|s| s.number == number) {
            let seg = self.cache.remove(pos).unwrap();
            self.total_in_flight_size = self
                .total_in_flight_size
                .saturating_sub(seg.payload.len() as u32);
            if current >= seg.timestamp {
                return Some(current - seg.timestamp);
            }
        }
        None
    }

    /// Gathers timed-out segments for transmission / retransmission
    pub fn flush(
        &mut self,
        current: u32,
        rto: u32,
        max_in_flight_size: u32,
        to_send: &mut Vec<DataSegment>,
    ) {
        if self.is_empty() {
            return;
        }

        let mut sent_size = 0u32;
        for seg in &mut self.cache {
            let mut need_send = false;
            if seg.transmit == 0 {
                need_send = true;
                seg.transmit = 1;
                seg.timeout = current + rto;
                seg.timestamp = current;
            } else if current >= seg.timeout {
                need_send = true;
                seg.transmit += 1;
                let step = rto.max(30);
                seg.timeout = current + step * (1 << (seg.transmit - 1).min(4));
            }

            if need_send {
                to_send.push(seg.clone());
                sent_size += seg.payload.len() as u32;
                if max_in_flight_size > 0 && sent_size >= max_in_flight_size {
                    break;
                }
            }
        }
    }
}

impl Default for SendingWindow {
    fn default() -> Self {
        Self::new()
    }
}
