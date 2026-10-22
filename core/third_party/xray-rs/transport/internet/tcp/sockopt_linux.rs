// Module: transport\internet\tcp\sockopt_linux.rs
// 1:1 Rust implementation corresponding to Go transport\internet\tcp\sockopt_linux.go

pub const TCP_FASTOPEN_CONNECT: i32 = 30;

pub fn is_fastopen_supported() -> bool {
    true
}
