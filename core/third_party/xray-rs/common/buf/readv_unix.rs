// Module: common\buf\readv_unix.rs
// 1:1 Rust implementation corresponding to Go common\buf\readv_unix.go

pub use super::readv_windows::WindowsVectorReader as UnixVectorReader;
