// Module: proxy\wireguard\tun_linux.rs
// 1:1 Rust implementation corresponding to Go proxy\wireguard\tun_linux.go

pub fn is_linux_kernel_wireguard_supported() -> bool {
    cfg!(target_os = "linux")
}
