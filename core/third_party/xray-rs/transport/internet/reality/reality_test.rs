// Module: transport\internet\reality\reality_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\reality\reality.go

#[cfg(test)]
mod tests {
    use super::super::config::RealityConfig;
    use super::super::reality::{
        RealityClient, RealityServer, derive_auth_key, open_session_id, seal_session_id,
    };
    use rand::RngCore;
    use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};

    #[test]
    fn test_reality_ecdh_hkdf_session_id_roundtrip() {
        let server_secret = StaticSecret::random_from_rng(rand::thread_rng());
        let server_pub = PublicKey::from(&server_secret);

        let client_secret = EphemeralSecret::random_from_rng(rand::thread_rng());
        let client_pub = PublicKey::from(&client_secret);

        let shared_client = client_secret.diffie_hellman(&server_pub);
        let shared_server = server_secret.diffie_hellman(&client_pub);
        assert_eq!(shared_client.as_bytes(), shared_server.as_bytes());

        let mut client_random = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut client_random);

        let auth_key_client = derive_auth_key(shared_client.as_bytes(), &client_random[..20]);
        let auth_key_server = derive_auth_key(shared_server.as_bytes(), &client_random[..20]);
        assert_eq!(auth_key_client, auth_key_server);

        let client_hello_raw = b"synthetic-client-hello-raw-handshake-bytes";
        let timestamp = 1700000000;
        let short_id = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];

        let sealed = seal_session_id(
            &auth_key_client,
            &client_random[20..],
            (1, 26, 3),
            timestamp,
            &short_id,
            client_hello_raw,
        )
        .expect("Seal session ID");

        let opened = open_session_id(
            &auth_key_server,
            &client_random[20..],
            &sealed,
            client_hello_raw,
        )
        .expect("Open session ID");

        assert_eq!(opened.version, (1, 26, 3));
        assert_eq!(opened.timestamp, timestamp);
        assert_eq!(opened.short_id, short_id.to_vec());
    }

    #[test]
    fn test_reality_client_server_full_auth_flow() {
        let server_secret = StaticSecret::random_from_rng(rand::thread_rng());
        let server_pub = PublicKey::from(&server_secret);
        let server_pub_hex = hex::encode(server_pub.as_bytes());
        let server_priv_hex = hex::encode(server_secret.to_bytes());

        let short_id = "0102030405060708";
        let config = RealityConfig {
            server_names: vec!["www.apple.com".into()],
            short_ids: vec![short_id.into()],
            private_key: Some(server_priv_hex),
            ..Default::default()
        };

        let server = RealityServer::new(&config).expect("Server new");
        let client =
            RealityClient::new("www.apple.com", &server_pub_hex, short_id).expect("Client new");

        let client_secret = EphemeralSecret::random_from_rng(rand::thread_rng());
        let client_pub = PublicKey::from(&client_secret);

        let mut client_random = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut client_random);

        let client_hello_raw = b"full-client-hello-message";
        let session_id = client
            .auth_client_hello(client_secret, &client_random, client_hello_raw)
            .expect("Auth client hello");

        let verified = server
            .verify_client_hello(
                client_pub.as_bytes(),
                &client_random,
                &session_id,
                client_hello_raw,
                30,
            )
            .expect("Verify client hello");

        assert_eq!(verified.version, (1, 26, 3));
        assert!(server.validate_short_id(&verified.short_id));
        assert!(server.validate_server_name("www.apple.com"));
    }

    #[test]
    fn test_reality_tampered_aad_fails() {
        let server_secret = StaticSecret::random_from_rng(rand::thread_rng());
        let server_pub = PublicKey::from(&server_secret);
        let client_secret = EphemeralSecret::random_from_rng(rand::thread_rng());
        let client_pub = PublicKey::from(&client_secret);

        let client = RealityClient::new(
            "www.apple.com",
            &hex::encode(server_pub.as_bytes()),
            "0102030405060708",
        )
        .unwrap();

        let mut client_random = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut client_random);

        let original_aad = b"original-client-hello";
        let session_id = client
            .auth_client_hello(client_secret, &client_random, original_aad)
            .unwrap();

        let server = RealityServer::new(&RealityConfig {
            private_key: Some(hex::encode(server_secret.to_bytes())),
            short_ids: vec!["0102030405060708".into()],
            ..Default::default()
        })
        .unwrap();

        let tampered_aad = b"tampered-client-hello";
        let result = server.verify_client_hello(
            client_pub.as_bytes(),
            &client_random,
            &session_id,
            tampered_aad,
            30,
        );
        assert!(
            result.is_err(),
            "Tampered AAD must fail AEAD authentication"
        );
    }
}
