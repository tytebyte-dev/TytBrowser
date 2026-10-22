// Module: proxy\wireguard\tun.rs
// 1:1 Rust implementation corresponding to Go proxy\wireguard\tun.go

pub struct WireGuardTunDevice {
    pub name: String,
    pub mtu: u32,
}

impl WireGuardTunDevice {
    pub fn new(name: impl Into<String>, mtu: u32) -> Self {
        Self {
            name: name.into(),
            mtu,
        }
    }
}
