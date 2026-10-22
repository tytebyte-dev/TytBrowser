// Module: common\dice\dice.rs
// 1:1 Rust implementation corresponding to Go common\dice\dice.go

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Roll returns a non-negative number between 0 (inclusive) and n (exclusive).
pub fn roll(n: usize) -> usize {
    if n <= 1 {
        return 0;
    }
    rand::thread_rng().gen_range(0..n)
}

/// RollInt63n returns a non-negative number between 0 (inclusive) and n (exclusive).
pub fn roll_int63n(n: i64) -> i64 {
    if n <= 1 {
        return 0;
    }
    rand::thread_rng().gen_range(0..n)
}

/// RollDeterministic returns a non-negative number between 0 (inclusive) and n (exclusive) with deterministic seed.
pub fn roll_deterministic(n: usize, seed: i64) -> usize {
    if n <= 1 {
        return 0;
    }
    let mut rng = StdRng::seed_from_u64(seed as u64);
    rng.gen_range(0..n)
}

/// RollUint16 returns a random uint16 value.
pub fn roll_uint16() -> u16 {
    rand::thread_rng().r#gen()
}

/// RollUint64 returns a random uint64 value.
pub fn roll_uint64() -> u64 {
    rand::thread_rng().r#gen()
}

pub use roll_uint16 as roll_u16;
pub use roll_uint64 as roll_u64;

/// DeterministicDice provides deterministic random number generation from a seed.
pub struct DeterministicDice {
    rng: StdRng,
}

impl DeterministicDice {
    pub fn new(seed: i64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed as u64),
        }
    }

    pub fn roll(&mut self, n: usize) -> usize {
        if n <= 1 {
            return 0;
        }
        self.rng.gen_range(0..n)
    }
}
