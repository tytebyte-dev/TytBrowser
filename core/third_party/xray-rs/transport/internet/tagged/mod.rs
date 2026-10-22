// Module: transport\internet\tagged\mod.rs
// 1:1 Rust implementation corresponding to Go transport\internet\tagged

pub mod tagged;
pub mod taggedimpl;

#[cfg(test)]
pub mod tagged_test;

pub use tagged::{DialFunc, TaggedDialer, dial, set_dialer};
