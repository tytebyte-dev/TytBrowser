// Module: proxy\trojan\trojan.rs
// 1:1 Rust implementation corresponding to Go proxy\trojan\trojan.go

pub const PROTOCOL_NAME: &str = "trojan";
pub const CRLF: &[u8; 2] = b"\r\n";
pub const COMMAND_TCP: u8 = 0x01;
pub const COMMAND_UDP: u8 = 0x03;
pub const COMMAND_MUX: u8 = 0x7f;
