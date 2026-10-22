#[cfg(test)]
mod tests {
    use crate::common::net::{Address, Destination};
    use crate::common::protocol::RequestCommand;
    use crate::proxy::vless::encoding::{Addons, RequestHeader, ResponseHeader};
    use std::io::Cursor;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_vless_addons_protobuf_roundtrip() {
        let addons = Addons {
            flow: "xtls-rprx-vision".into(),
            seed: vec![1, 2, 3, 4, 5, 6, 7, 8],
        };
        let encoded = addons.encode();
        let decoded = Addons::decode(&encoded);

        assert_eq!(decoded.flow, "xtls-rprx-vision");
        assert_eq!(decoded.seed, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[tokio::test]
    async fn test_vless_request_and_response_header_roundtrip() {
        let user_id = Uuid::new_v4();
        let target = Destination::tcp(Address::Domain("cloudflare.com".into()), 443);
        let addons = Addons {
            flow: "xtls-rprx-vision".into(),
            seed: vec![0xaa, 0xbb],
        };

        let req = RequestHeader {
            user_id,
            command: RequestCommand::Tcp,
            destination: target.clone(),
            addons: addons.clone(),
        };

        let mut buf = Vec::new();
        req.encode(&mut buf).await.unwrap();

        let mut cursor = Cursor::new(buf);
        let dec_req = RequestHeader::decode(&mut cursor).await.unwrap();

        assert_eq!(dec_req.user_id, user_id);
        assert_eq!(dec_req.command, RequestCommand::Tcp);
        assert_eq!(dec_req.destination, target);
        assert_eq!(dec_req.addons.flow, "xtls-rprx-vision");
        assert_eq!(dec_req.addons.seed, vec![0xaa, 0xbb]);

        // Response header test
        let resp = ResponseHeader { addons };
        let mut resp_buf = Vec::new();
        resp.encode(&mut resp_buf).await.unwrap();

        let mut resp_cursor = Cursor::new(resp_buf);
        let dec_resp = ResponseHeader::decode(&mut resp_cursor).await.unwrap();
        assert_eq!(dec_resp.addons.flow, "xtls-rprx-vision");
    }
}
