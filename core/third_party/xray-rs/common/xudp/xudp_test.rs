// Module: common\xudp\xudp_test.rs
// 1:1 Rust unit test suite corresponding to Go common\xudp\xudp_test.go

#[cfg(test)]
mod tests {
    use super::super::{
        PacketReader, PacketWriter, XUDP_MAGIC, XudpPacket, generate_global_id, read_address_port,
        write_address_port,
    };
    use crate::common::net::{Address, Destination};
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
    use tokio::io::duplex;

    #[test]
    fn test_xudp_magic() {
        assert_eq!(XUDP_MAGIC, 0x5855);
    }

    #[test]
    fn test_global_id_generation() {
        let addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();
        let base_key = [0x42u8; 32];
        let id1 = generate_global_id(&addr, &base_key);
        let id2 = generate_global_id(&addr, &base_key);
        assert_eq!(id1, id2);

        let other_addr: SocketAddr = "127.0.0.1:12346".parse().unwrap();
        let id3 = generate_global_id(&other_addr, &base_key);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_address_port_codec_roundtrip() {
        // IPv4
        let mut buf = Vec::new();
        let ipv4 = Address::Ipv4(Ipv4Addr::new(192, 168, 1, 100));
        write_address_port(&mut buf, &ipv4, 8080);
        let (parsed_addr, parsed_port, consumed) = read_address_port(&buf).unwrap();
        assert_eq!(parsed_addr, ipv4);
        assert_eq!(parsed_port, 8080);
        assert_eq!(consumed, buf.len());

        // Domain
        let mut buf_dom = Vec::new();
        let domain = Address::Domain("example.com".to_string());
        write_address_port(&mut buf_dom, &domain, 443);
        let (parsed_dom, parsed_dport, d_consumed) = read_address_port(&buf_dom).unwrap();
        assert_eq!(parsed_dom, domain);
        assert_eq!(parsed_dport, 443);
        assert_eq!(d_consumed, buf_dom.len());

        // IPv6
        let mut buf_v6 = Vec::new();
        let ipv6 = Address::Ipv6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1));
        write_address_port(&mut buf_v6, &ipv6, 53);
        let (parsed_v6, parsed_v6port, v6_consumed) = read_address_port(&buf_v6).unwrap();
        assert_eq!(parsed_v6, ipv6);
        assert_eq!(parsed_v6port, 53);
        assert_eq!(v6_consumed, buf_v6.len());
    }

    #[tokio::test]
    async fn test_packet_writer_and_reader_streaming() {
        let (client, server) = duplex(4096);
        let dest = Destination::udp(Address::Domain("dns.google".to_string()), 53);
        let global_id = [1, 2, 3, 4, 5, 6, 7, 8];

        let mut writer = PacketWriter::new(client, dest.clone(), global_id);
        let mut reader = PacketReader::new(server);

        let writer_task = tokio::spawn(async move {
            writer.write_packet(b"query-payload-1", true).await.unwrap();
            writer
                .write_packet(b"query-payload-2", false)
                .await
                .unwrap();
        });

        let reader_task = tokio::spawn(async move {
            let (dest1, payload1) = reader.read_packet().await.unwrap();
            assert_eq!(dest1.port, 53);
            assert_eq!(payload1, b"query-payload-1");

            let (dest2, payload2) = reader.read_packet().await.unwrap();
            assert_eq!(dest2.port, 53);
            assert_eq!(payload2, b"query-payload-2");
        });

        writer_task.await.unwrap();
        reader_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_xudp_packet_encode_decode() {
        let (mut client, mut server) = duplex(1024);
        let dest = Destination::udp(Address::Ipv4(Ipv4Addr::new(8, 8, 8, 8)), 53);
        let packet = XudpPacket::new(dest.clone(), b"dns-req".to_vec());

        let writer_task = tokio::spawn(async move {
            packet.encode(&mut client).await.unwrap();
        });

        let reader_task = tokio::spawn(async move {
            let decoded = XudpPacket::decode(&mut server).await.unwrap();
            assert_eq!(decoded.destination, dest);
            assert_eq!(decoded.payload, b"dns-req");
        });

        writer_task.await.unwrap();
        reader_task.await.unwrap();
    }
}
