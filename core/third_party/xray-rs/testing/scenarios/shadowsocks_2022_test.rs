#[cfg(test)]
mod tests {
    use crate::common::net::{Address, Destination};
    use crate::proxy::shadowsocks_2022::SessionHeader;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_ss2022_session_header_roundtrip() {
        let dest = Destination::new(Address::Domain("api.openai.com".into()), 443);
        let header = SessionHeader::new_client(dest.clone());

        let mut buf = Vec::new();
        header.encode(&mut buf).await.unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded = SessionHeader::decode(&mut cursor).await.unwrap();

        assert_eq!(decoded.header_type, header.header_type);
        assert_eq!(decoded.timestamp, header.timestamp);
        assert_eq!(decoded.session_id, header.session_id);
        assert_eq!(decoded.destination, Some(dest));
    }
}
