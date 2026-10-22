// Module: transport\internet\tcp\sockopt_freebsd.rs
// 1:1 Rust implementation corresponding to Go transport\internet\tcp\sockopt_freebsd.go

pub const TCP_FASTOPEN_CONNECT: i32 = 0x105;

pub fn is_fastopen_supported() -> bool {
    false
}
