// Module: common\protocol\udp\packet.rs
// 1:1 Rust implementation corresponding to Go common\protocol\udp\packet.go

use crate::common::buf::Buffer;
use crate::common::net::Destination;

#[derive(Debug, Clone)]
pub struct UdpPacket {
    pub payload: Buffer,
    pub source: Destination,
    pub target: Destination,
}

impl UdpPacket {
    pub fn new(payload: Buffer, source: Destination, target: Destination) -> Self {
        Self {
            payload,
            source,
            target,
        }
    }
}
