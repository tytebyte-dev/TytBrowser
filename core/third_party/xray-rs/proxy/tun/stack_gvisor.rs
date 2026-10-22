// Module: proxy\tun\stack_gvisor.rs
// 1:1 Rust implementation corresponding to Go proxy\tun\stack_gvisor.go

pub struct GVisorStack {
    pub mtu: u32,
}

impl GVisorStack {
    pub fn new() -> Self {
        Self { mtu: 1500 }
    }

    pub fn with_mtu(mtu: u32) -> Self {
        Self { mtu }
    }
}

impl Default for GVisorStack {
    fn default() -> Self {
        Self::new()
    }
}

pub type GVisorTunStack = GVisorStack;
