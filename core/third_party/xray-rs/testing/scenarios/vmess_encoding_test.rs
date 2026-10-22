#[cfg(test)]
mod tests {
    use crate::common::net::{Address, Destination};
    use crate::common::protocol::RequestCommand;
    use crate::proxy::vmess::encoding::{
        RequestHeader, ResponseHeader, fnv1a_32, generate_chacha20_key,
    };
    use std::io::Cursor;
    use uuid::Uuid;

    #[test]
    fn test_vmess_fnv1a_and_key_generation() {
        let test_data = b"VMess AEAD Auth Data";
        let hash = fnv1a_32(test_data);
        assert_ne!(hash, 0);

        let seed = [0x7au8; 16];
        let k32 = generate_chacha20_key(&seed);
        assert_eq!(k32.len(), 32);
        assert_ne!(&k32[0..16], &k32[16..32]);
    }

    #[tokio::test]
    async fn test_vmess_request_and_response_header_roundtrip() {
        let user_id = Uuid::new_v4();
        let target = Destination::tcp(Address::Domain("fast.com".into()), 443);
        let header = RequestHeader::new(user_id, RequestCommand::Tcp, target.clone());

        let mut buf = Vec::new();
        header.encode(&mut buf).await.unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded = RequestHeader::decode(&mut cursor).await.unwrap();

        assert_eq!(decoded.command, RequestCommand::Tcp);
        assert_eq!(decoded.destination, target);
        assert_eq!(decoded.request_body_iv, header.request_body_iv);
        assert_eq!(decoded.request_body_key, header.request_body_key);

        // Response header test
        let resp = ResponseHeader::new(0x37);
        let mut resp_buf = Vec::new();
        resp.encode(&mut resp_buf).await.unwrap();

        let mut resp_cursor = Cursor::new(resp_buf);
        let dec_resp = ResponseHeader::decode(&mut resp_cursor).await.unwrap();
        assert_eq!(dec_resp.response_header_byte, 0x37);
    }
}
