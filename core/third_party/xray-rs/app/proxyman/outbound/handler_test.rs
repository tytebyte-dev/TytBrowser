// Module: app\proxyman\outbound\handler_test.rs
// 1:1 Rust unit test suite corresponding to Go app\proxyman\outbound\handler_test.go

#[cfg(test)]
mod tests {
    use super::super::{
        DefaultOutboundHandler, UOT_LEGACY_MAGIC_ADDRESS, UOT_MAGIC_ADDRESS, UotPacket, UotVersion,
        is_uot_destination,
    };
    use crate::common::net::{Address, Destination};
    use crate::features::outbound::OutboundHandler;
    use std::net::Ipv4Addr;

    #[test]
    fn test_outbound_handler_tag() {
        let handler = DefaultOutboundHandler::new("direct");
        assert_eq!(handler.tag(), "direct");
    }

    #[test]
    fn test_uot_magic_destination_detection() {
        let std_dest = Destination::udp(Address::Domain(UOT_MAGIC_ADDRESS.into()), 53);
        assert_eq!(is_uot_destination(&std_dest), Some(UotVersion::Standard));

        let legacy_dest = Destination::udp(Address::Domain(UOT_LEGACY_MAGIC_ADDRESS.into()), 53);
        assert_eq!(is_uot_destination(&legacy_dest), Some(UotVersion::Legacy));

        let normal_dest = Destination::udp(Address::Ipv4(Ipv4Addr::new(8, 8, 8, 8)), 53);
        assert_eq!(is_uot_destination(&normal_dest), None);
    }

    #[test]
    fn test_uot_packet_encode_decode_ipv4() {
        let dest = Destination::udp(Address::Ipv4(Ipv4Addr::new(1, 1, 1, 1)), 53);
        let payload = vec![0x12, 0x34, 0x56, 0x78];
        let packet = UotPacket::new(dest, payload.clone());

        let encoded = packet.encode(UotVersion::Standard);
        let (decoded, consumed) =
            UotPacket::decode(&encoded, UotVersion::Standard).expect("decode UoT packet");

        assert_eq!(consumed, encoded.len());
        assert_eq!(decoded.destination.port, 53);
        assert_eq!(
            decoded.destination.address,
            Address::Ipv4(Ipv4Addr::new(1, 1, 1, 1))
        );
        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn test_uot_packet_encode_decode_domain() {
        let dest = Destination::udp(Address::Domain("dns.google".into()), 53);
        let payload = b"query-data".to_vec();
        let packet = UotPacket::new(dest, payload.clone());

        let encoded = packet.encode(UotVersion::Standard);
        let (decoded, consumed) =
            UotPacket::decode(&encoded, UotVersion::Standard).expect("decode domain UoT");

        assert_eq!(consumed, encoded.len());
        assert_eq!(decoded.destination.port, 53);
        assert_eq!(
            decoded.destination.address,
            Address::Domain("dns.google".into())
        );
        assert_eq!(decoded.payload, payload);
    }
}
