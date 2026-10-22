#[cfg(test)]
mod tests {
    use crate::common::net::{Address, Destination};
    use crate::common::protocol::RequestCommand;
    use crate::proxy::trojan::protocol::{TrojanRequestHeader, TrojanUdpPacket, hash_password};
    use std::io::Cursor;

    #[tokio::test]
    async fn test_trojan_tcp_request_header_roundtrip() {
        let password = "my_secure_trojan_password";
        let target = Destination::tcp(Address::Domain("api.github.com".into()), 443);

        let header = TrojanRequestHeader::new(password, RequestCommand::Tcp, target.clone());

        let mut buf = Vec::new();
        header.encode(&mut buf).await.unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded = TrojanRequestHeader::decode(&mut cursor).await.unwrap();

        assert_eq!(decoded.password_hash, hash_password(password));
        assert_eq!(decoded.command, RequestCommand::Tcp);
        assert_eq!(decoded.destination, target);
    }

    #[tokio::test]
    async fn test_trojan_udp_packet_roundtrip() {
        let target = Destination::udp(Address::Ipv4(std::net::Ipv4Addr::new(8, 8, 8, 8)), 53);
        let payload = b"DNS standard query 0x1234";

        let packet = TrojanUdpPacket::new(target.clone(), payload.to_vec());

        let mut buf = Vec::new();
        packet.encode(&mut buf).await.unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded = TrojanUdpPacket::decode(&mut cursor).await.unwrap();

        assert_eq!(decoded.destination, target);
        assert_eq!(decoded.payload, payload);
    }
}
