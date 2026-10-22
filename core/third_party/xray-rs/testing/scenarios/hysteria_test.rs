#[cfg(test)]
mod tests {
    use crate::proxy::hysteria::{QuicVarint, TcpRequest};
    use std::io::Cursor;

    #[tokio::test]
    async fn test_quic_varint_roundtrip() {
        let test_vals = vec![0, 1, 63, 64, 16383, 16384, 1073741823, 1073741824];
        for val in test_vals {
            let mut buf = Vec::new();
            QuicVarint::write(val, &mut buf);
            let mut cursor = Cursor::new(buf);
            let decoded = QuicVarint::read(&mut cursor).await.unwrap();
            assert_eq!(decoded, val);
        }
    }

    #[tokio::test]
    async fn test_hysteria_tcp_request_roundtrip() {
        let req = TcpRequest::new("google.com:443", 32);
        let encoded = req.encode();

        let mut cursor = Cursor::new(encoded);
        let decoded = TcpRequest::decode(&mut cursor).await.unwrap();
        assert_eq!(decoded.address, "google.com:443");
        assert_eq!(decoded.padding_len, 32);
    }
}
