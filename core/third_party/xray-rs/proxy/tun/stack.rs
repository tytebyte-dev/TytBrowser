// Module: proxy\tun\stack.rs
// 1:1 Rust implementation corresponding to Go proxy\tun\stack.go

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TunStack {
    #[default]
    System,
    GVisor,
    Mixed,
}
