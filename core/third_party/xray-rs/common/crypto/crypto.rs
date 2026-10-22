// Module: common\crypto\crypto.rs
// 1:1 Rust implementation corresponding to Go common\crypto\crypto.go

use crate::common::errors::Result;
use rand::Rng;

pub trait StreamCipher: Send + Sync {
    fn encrypt(&mut self, buffer: &mut [u8]) -> Result<()>;
    fn decrypt(&mut self, buffer: &mut [u8]) -> Result<()>;
}

/// Returns a random i64 in range [from, to).
/// 1:1 corresponding to RandBetween in crypto.go.
pub fn rand_between(mut from: i64, mut to: i64) -> i64 {
    if from == to {
        return from;
    }
    if from > to {
        std::mem::swap(&mut from, &mut to);
    }
    let mut rng = rand::thread_rng();
    rng.gen_range(from..to)
}

/// Fills slice b with bytes between from and to inclusive.
/// 1:1 corresponding to RandBytesBetween in crypto.go.
pub fn rand_bytes_between(b: &mut [u8], mut from: u8, mut to: u8) {
    if from > to {
        std::mem::swap(&mut from, &mut to);
    }
    let mut rng = rand::thread_rng();
    rng.fill(b);

    if to.wrapping_sub(from) == 255 {
        return;
    }

    let range = (to - from + 1) as usize;
    for byte in b.iter_mut() {
        *byte = from + ((*byte as usize) % range) as u8;
    }
}
