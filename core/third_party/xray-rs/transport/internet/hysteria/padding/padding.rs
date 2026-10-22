// Module: transport\internet\hysteria\padding\padding.rs
// 1:1 Rust implementation corresponding to Go transport\internet\hysteria\padding\padding.go

use rand::Rng;

pub const PADDING_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Padding {
    pub min: usize,
    pub max: usize,
}

impl Padding {
    pub fn new(min: usize, max: usize) -> Self {
        Self { min, max }
    }

    pub fn generate_string(&self) -> String {
        if self.min >= self.max {
            return String::new();
        }
        let mut rng = rand::thread_rng();
        let n = rng.gen_range(self.min..self.max);
        let mut bs = Vec::with_capacity(n);
        for _ in 0..n {
            let idx = rng.gen_range(0..PADDING_CHARS.len());
            bs.push(PADDING_CHARS[idx]);
        }
        String::from_utf8(bs).unwrap_or_default()
    }
}

pub fn generate_padding(len: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..len).map(|_| rng.r#gen()).collect()
}
