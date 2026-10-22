#[cfg(test)]
mod tests {
    use crate::common::net::{Address, Destination};
    use crate::common::serial::{
        concat_strings, read_u16, read_u32, read_u64, write_u16, write_u32, write_u64,
    };
    use crate::common::xudp::{XudpPacket, generate_global_id};
    use std::io::Cursor;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_serial_integer_and_string_operations() {
        let mut buf = Vec::new();
        write_u16(&mut buf, 0x1234).await.unwrap();
        write_u32(&mut buf, 0x56789abc).await.unwrap();
        write_u64(&mut buf, 0x1122334455667788).await.unwrap();

        let mut cursor = Cursor::new(buf);
        assert_eq!(read_u16(&mut cursor).await.unwrap(), 0x1234);
        assert_eq!(read_u32(&mut cursor).await.unwrap(), 0x56789abc);
        assert_eq!(read_u64(&mut cursor).await.unwrap(), 0x1122334455667788);

        let merged = concat_strings(&["hello", " ", "world"]);
        assert_eq!(merged, "hello world");
    }

    #[tokio::test]
    async fn test_xudp_packet_and_global_id() {
        let addr: SocketAddr = "127.0.0.1:54321".parse().unwrap();
        let base_key = [0x55u8; 32];
        let gid = generate_global_id(&addr, &base_key);
        assert_ne!(gid, [0u8; 8]);

        let target = Destination::udp(Address::Domain("dns.quad9.net".into()), 53);
        let payload = b"query xudp payload";
        let packet = XudpPacket::new(target.clone(), payload.to_vec());

        let mut buf = Vec::new();
        packet.encode(&mut buf).await.unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded = XudpPacket::decode(&mut cursor).await.unwrap();

        assert_eq!(decoded.destination, target);
        assert_eq!(decoded.payload, payload);
    }
}
