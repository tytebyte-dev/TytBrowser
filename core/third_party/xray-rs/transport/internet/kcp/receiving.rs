// Module: transport\internet\kcp\receiving.rs
// 1:1 Rust implementation corresponding to Go transport\internet\kcp\receiving.go

use super::segment::{ACK_NUMBER_LIMIT, AckSegment, DataSegment};
use std::collections::BTreeMap;

pub struct ReceivingWindow {
    pub cache: BTreeMap<u32, DataSegment>,
    pub next_number: u32,
}

impl ReceivingWindow {
    pub fn new() -> Self {
        Self {
            cache: BTreeMap::new(),
            next_number: 0,
        }
    }

    pub fn set(&mut self, seg: DataSegment) -> bool {
        let id = seg.number;
        if id.wrapping_sub(self.next_number) >= 0x7FFFFFFF {
            // Already received and passed
            return false;
        }
        if self.cache.contains_key(&id) {
            return false;
        }
        self.cache.insert(id, seg);
        true
    }

    pub fn has(&self, id: u32) -> bool {
        self.cache.contains_key(&id)
    }

    pub fn remove(&mut self, id: u32) -> Option<DataSegment> {
        self.cache.remove(&id)
    }

    /// Drains contiguous in-order payloads starting from self.next_number into user buffer
    pub fn drain_in_order(&mut self, user_buf: &mut Vec<u8>) -> usize {
        let mut total = 0;
        while let Some(seg) = self.cache.remove(&self.next_number) {
            total += seg.payload.len();
            user_buf.extend_from_slice(&seg.payload);
            self.next_number = self.next_number.wrapping_add(1);
        }
        total
    }
}

impl Default for ReceivingWindow {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AckList {
    pub numbers: Vec<u32>,
    pub timestamps: Vec<u32>,
    pub dirty: bool,
}

impl AckList {
    pub fn new() -> Self {
        Self {
            numbers: Vec::with_capacity(128),
            timestamps: Vec::with_capacity(128),
            dirty: false,
        }
    }

    pub fn add(&mut self, number: u32, timestamp: u32) {
        self.numbers.push(number);
        self.timestamps.push(timestamp);
        self.dirty = true;
    }

    pub fn clear(&mut self, una: u32) {
        let mut new_nums = Vec::with_capacity(self.numbers.len());
        let mut new_ts = Vec::with_capacity(self.timestamps.len());

        for i in 0..self.numbers.len() {
            // keep if number >= una
            if self.numbers[i].wrapping_sub(una) < 0x7FFFFFFF {
                new_nums.push(self.numbers[i]);
                new_ts.push(self.timestamps[i]);
            }
        }

        if new_nums.len() < self.numbers.len() {
            self.dirty = true;
        }
        self.numbers = new_nums;
        self.timestamps = new_ts;
    }

    pub fn flush(
        &mut self,
        conv: u16,
        rcv_wnd: u32,
        rcv_next: u32,
        current: u32,
    ) -> Option<AckSegment> {
        if !self.dirty && self.numbers.is_empty() {
            return None;
        }

        let mut seg = AckSegment::new(conv, ACK_NUMBER_LIMIT);
        seg.receiving_window = rcv_wnd;
        seg.receiving_next = rcv_next;
        seg.timestamp = current;

        for &num in self.numbers.iter().take(ACK_NUMBER_LIMIT) {
            seg.put_number(num);
        }

        self.dirty = false;
        Some(seg)
    }
}

impl Default for AckList {
    fn default() -> Self {
        Self::new()
    }
}
