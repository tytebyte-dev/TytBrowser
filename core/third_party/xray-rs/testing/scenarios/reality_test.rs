#[cfg(test)]
mod tests {
    use crate::transport::internet::reality::{RealityClient, RealityConfig, RealityServer};

    #[test]
    fn test_reality_server_and_client_auth() {
        let config = RealityConfig {
            show: false,
            dest: Some("www.apple.com:443".into()),
            server_names: vec!["www.apple.com".into()],
            private_key: Some(
                "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".into(),
            ),
            public_key: None,
            min_client_ver: None,
            max_client_ver: None,
            max_time_diff: None,
            short_ids: vec!["0123456789abcdef".into()],
            spider_x: None,
        };

        let server = RealityServer::new(&config).unwrap();
        assert!(server.validate_server_name("www.apple.com"));
        assert!(!server.validate_server_name("www.google.com"));

        let short_id_bytes = hex::decode("0123456789abcdef").unwrap();
        assert!(server.validate_short_id(&short_id_bytes));
        assert!(!server.validate_short_id(&[0u8; 8]));

        let client = RealityClient::new(
            "www.apple.com",
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
            "0123456789abcdef",
        )
        .unwrap();

        let (client_pub, shared_secret) = client.generate_auth();
        assert_eq!(client_pub.len(), 32);
        assert_eq!(shared_secret.len(), 32);
    }
}
