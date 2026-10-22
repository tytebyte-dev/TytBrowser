// Module: proxy\proxy_test.rs
// Comprehensive unit tests for proxy core traits & XTLS Vision padding engine

#[cfg(test)]
mod tests {
    use crate::proxy::proxy::{
        COMMAND_PADDING_CONTINUE, COMMAND_PADDING_END, DefaultUserManager,
        TLS_APPLICATION_DATA_START, TLS_CLIENT_HANDSHAKE_START, TLS_SERVER_HANDSHAKE_START,
        TLS13_CIPHER_SUITE_DIC, TrafficState, UserManager, is_complete_record, xtls_filter_tls,
        xtls_padding, xtls_unpadding,
    };

    #[test]
    fn test_constants_and_cipher_dictionary() {
        assert_eq!(TLS_CLIENT_HANDSHAKE_START, &[0x16, 0x03]);
        assert_eq!(TLS_SERVER_HANDSHAKE_START, &[0x16, 0x03, 0x03]);
        assert_eq!(TLS_APPLICATION_DATA_START, &[0x17, 0x03, 0x03]);
        assert_eq!(
            TLS13_CIPHER_SUITE_DIC.get(&0x1301),
            Some(&"TLS_AES_128_GCM_SHA256")
        );
        assert_eq!(
            TLS13_CIPHER_SUITE_DIC.get(&0x1303),
            Some(&"TLS_CHACHA20_POLY1305_SHA256")
        );
    }

    #[test]
    fn test_xtls_padding_and_unpadding_roundtrip() {
        let uuid = [0xabu8; 16];
        let mut traffic_state = TrafficState::new(&uuid);

        let content = b"Hello XTLS Vision Protocol Payload 1234567890!";
        let testseed = [900, 100, 900, 64];

        // 1. Pack with padding continue and user UUID
        let padded = xtls_padding(
            Some(content),
            COMMAND_PADDING_CONTINUE,
            Some(&uuid),
            false,
            Some(&testseed),
        );

        assert!(padded.len() > content.len() + 16 + 5);

        // 2. Unpack uplink
        let unpacked = xtls_unpadding(&padded, &mut traffic_state, true);
        assert_eq!(&unpacked, content);

        // 3. Pack second block with padding end (without UUID)
        let content2 = b"Second vision payload block";
        let padded2 = xtls_padding(
            Some(content2),
            COMMAND_PADDING_END,
            None,
            false,
            Some(&testseed),
        );

        let unpacked2 = xtls_unpadding(&padded2, &mut traffic_state, true);
        assert_eq!(&unpacked2, content2);
    }

    #[test]
    fn test_is_complete_record() {
        // Incomplete header
        assert!(!is_complete_record(&[0x17, 0x03]));

        // Valid single record: 0x17, 0x03, 0x03, length = 4 (0x00, 0x04), payload = 4 bytes
        let valid_record = vec![0x17, 0x03, 0x03, 0x00, 0x04, 0x01, 0x02, 0x03, 0x04];
        assert!(is_complete_record(&valid_record));

        // Truncated record
        let truncated = vec![0x17, 0x03, 0x03, 0x00, 0x04, 0x01, 0x02];
        assert!(!is_complete_record(&truncated));
    }

    #[test]
    fn test_xtls_filter_tls() {
        let uuid = [0xabu8; 16];
        let mut traffic_state = TrafficState::new(&uuid);

        // ClientHello packet: 0x16, 0x03, 0x01, len_hi, len_lo, type=0x01
        let client_hello = vec![0x16, 0x03, 0x01, 0x00, 0x10, 0x01, 0x00, 0x00];
        xtls_filter_tls(&client_hello, &mut traffic_state);
        assert!(traffic_state.is_tls);
        assert_eq!(traffic_state.number_of_packet_to_filter, 7);
    }

    #[test]
    fn test_user_manager() {
        let mgr = DefaultUserManager::new();
        assert_eq!(mgr.get_users_count(), 0);

        assert!(mgr.add_user("test@example.com", 1).is_ok());
        assert_eq!(mgr.get_users_count(), 1);

        assert!(mgr.remove_user("test@example.com").is_ok());
        assert_eq!(mgr.get_users_count(), 0);
    }
}
