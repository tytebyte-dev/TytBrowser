// Module: common\protocol\payload.rs
// 1:1 Rust implementation corresponding to Go common\protocol\payload.go

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferType {
    Stream = 0,
    Packet = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressType {
    IPv4 = 1,
    Domain = 2,
    IPv6 = 3,
}
