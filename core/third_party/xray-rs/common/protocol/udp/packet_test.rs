// Module: common\protocol\udp\packet_test.rs
// 1:1 Rust unit test suite corresponding to Go common\protocol\udp\packet_test.go

#[cfg(test)]
mod tests {
    use crate::common::buf::Buffer;
    use crate::common::net::{Address, Destination};
    use crate::common::protocol::udp::UdpPacket;

    #[test]
    fn test_udp_packet_construction() {
        let mut buf = Buffer::new();
        buf.write(b"dns-query-data").unwrap();

        let src = Destination::udp(Address::Domain("127.0.0.1".into()), 12345);
        let dst = Destination::udp(Address::Domain("8.8.8.8".into()), 53);

        let packet = UdpPacket::new(buf, src.clone(), dst.clone());
        assert_eq!(packet.payload.as_slice(), b"dns-query-data");
        assert_eq!(packet.source, src);
        assert_eq!(packet.target, dst);
    }
}
