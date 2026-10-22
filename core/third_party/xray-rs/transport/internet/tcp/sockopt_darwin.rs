// Module: transport\internet\tcp\sockopt_darwin.rs
// 1:1 Rust implementation corresponding to Go transport\internet\tcp\sockopt_darwin.go

pub const TCP_FASTOPEN_CONNECT: i32 = 0x105;

pub fn is_fastopen_supported() -> bool {
    true
}
