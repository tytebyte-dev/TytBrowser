// Module: proxy\tun\stack_gvisor_endpoint.rs
// 1:1 Rust implementation corresponding to Go proxy\tun\stack_gvisor_endpoint.go

pub struct GVisorEndpoint {
    pub channel_id: u64,
}

impl GVisorEndpoint {
    pub fn new() -> Self {
        Self { channel_id: 1 }
    }

    pub fn id(&self) -> u64 {
        self.channel_id
    }
}

impl Default for GVisorEndpoint {
    fn default() -> Self {
        Self::new()
    }
}
