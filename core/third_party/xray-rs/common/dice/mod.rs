pub mod dice;

#[cfg(test)]
pub mod dice_test;

pub use dice::{
    DeterministicDice, roll, roll_deterministic, roll_int63n, roll_u16, roll_u64, roll_uint16,
    roll_uint64,
};
