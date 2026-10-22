// Module: common\buf\readv_posix.rs
// 1:1 Rust implementation corresponding to Go common\buf\readv_posix.go

pub use super::readv_windows::WindowsVectorReader as PosixVectorReader;
