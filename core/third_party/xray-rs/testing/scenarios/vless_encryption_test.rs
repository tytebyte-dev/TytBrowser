// Module: testing\scenarios\vless_encryption_test.rs
#[cfg(test)]
mod tests {
    use crate::proxy::vless::encryption::{VlessXorClient, VlessXorServer};

    #[test]
    fn test_vless_xor_roundtrip() {
        let client = VlessXorClient::new(vec![0xAA, 0xBB]);
        let server = VlessXorServer::new(vec![0xAA, 0xBB]);

        let mut data = b"encrypted message".to_vec();
        let orig = data.clone();
        client.encrypt(&mut data);
        assert_ne!(data, orig);
        server.decrypt(&mut data);
        assert_eq!(data, orig);
    }
}
