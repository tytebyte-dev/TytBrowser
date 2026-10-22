// Module: transport\internet\tagged\taggedimpl\mod.rs
// 1:1 Rust implementation corresponding to Go transport\internet\tagged\taggedimpl

#[path = "impl.rs"]
pub mod impl_;
pub mod taggedimpl;

pub use impl_::{dial_tagged_outbound, register_tagged_dialer};
