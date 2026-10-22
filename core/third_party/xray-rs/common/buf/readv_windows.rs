// Module: common\buf\readv_windows.rs
// 1:1 Rust implementation corresponding to Go common\buf\readv_windows.go

use crate::common::buf::Buffer;

pub struct WindowsVectorReader {
    bufs: Vec<Vec<u8>>,
}

impl WindowsVectorReader {
    pub fn new() -> Self {
        Self { bufs: Vec::new() }
    }

    pub fn init(&mut self, buffers: &[Buffer]) {
        self.bufs.clear();
        for b in buffers {
            self.bufs.push(b.as_slice().to_vec());
        }
    }

    pub fn clear(&mut self) {
        self.bufs.clear();
    }
}

impl Default for WindowsVectorReader {
    fn default() -> Self {
        Self::new()
    }
}
