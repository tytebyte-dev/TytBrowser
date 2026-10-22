// Module: proxy\wireguard\gvisortun\tun.rs
// 1:1 Rust implementation corresponding to Go proxy\wireguard\gvisortun\tun.go

pub struct GVisorWireguardTun {
    pub name: String,
}

impl GVisorWireguardTun {
    pub fn new() -> Self {
        Self { name: "wg0".into() }
    }

    pub fn device_name(&self) -> &str {
        &self.name
    }
}

impl Default for GVisorWireguardTun {
    fn default() -> Self {
        Self::new()
    }
}

pub type GVisorTunDevice = GVisorWireguardTun;
