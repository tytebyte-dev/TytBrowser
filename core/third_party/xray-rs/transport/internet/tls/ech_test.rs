// Module: transport\internet\tls\ech_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\tls\ech_test.go

#[cfg(test)]
mod tests {
    use super::super::ech::{
        DUMMY_FALLBACK_ECH_CONFIG, EchConfig, EchConfigCache, EchServerKey, GLOBAL_ECH_CACHE,
        convert_to_ech_keys, encode_ech_keys, encode_svcb_ech_param, parse_svcb_ech_param,
        resolve_ech_config,
    };
    use base64::Engine;
    use std::time::Duration;

    #[test]
    fn test_ech_config_construction() {
        let cfg = EchConfig::new(true, Some("cloudflare.com".into()));
        assert!(cfg.enabled);
        assert_eq!(cfg.outer_sni, Some("cloudflare.com".into()));
        assert_eq!(cfg.force_query, "full");
    }

    #[test]
    fn test_ech_keys_encode_decode_roundtrip() {
        let keys = vec![
            EchServerKey {
                private_key: vec![0x11, 0x22, 0x33, 0x44],
                config: vec![0xaa, 0xbb, 0xcc, 0xdd, 0xee],
            },
            EchServerKey {
                private_key: vec![0x55, 0x66],
                config: vec![0x77, 0x88, 0x99],
            },
        ];

        let encoded = encode_ech_keys(&keys);
        let decoded = convert_to_ech_keys(&encoded).expect("should decode cleanly");
        assert_eq!(decoded, keys);
    }

    #[test]
    fn test_ech_keys_invalid_length() {
        let truncated = vec![0x00, 0x10, 0x01, 0x02]; // claimed len 16, only 2 bytes
        let res = convert_to_ech_keys(&truncated);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "goech: invalid length");
    }

    #[test]
    fn test_svcb_ech_param_extraction() {
        let dummy_ech = vec![0x00, 0x40, 0xfe, 0x0d, 0x10, 0x20];
        let mut svcb_record = Vec::new();
        // Param 1: ALPN (key 1, len 3)
        svcb_record.extend_from_slice(&1u16.to_be_bytes());
        svcb_record.extend_from_slice(&3u16.to_be_bytes());
        svcb_record.extend_from_slice(b"h2\0");
        // Param 5: ECH
        let ech_param = encode_svcb_ech_param(&dummy_ech);
        svcb_record.extend_from_slice(&ech_param);
        // Param 4: IPv4 (key 4, len 4)
        svcb_record.extend_from_slice(&4u16.to_be_bytes());
        svcb_record.extend_from_slice(&4u16.to_be_bytes());
        svcb_record.extend_from_slice(&[1, 1, 1, 1]);

        let extracted = parse_svcb_ech_param(&svcb_record).expect("must find ECH param");
        assert_eq!(extracted, dummy_ech);

        // Missing param test
        let without_ech = svcb_record[..7].to_vec();
        assert!(parse_svcb_ech_param(&without_ech).is_none());
    }

    #[test]
    fn test_ech_cache_lifecycle() {
        let cache = EchConfigCache::new();
        let server = "udp://1.1.1.1";
        let domain = "encryptedsni.com";
        let test_config = vec![1, 2, 3, 4, 5, 6, 7, 8];

        assert!(cache.get(server, domain).is_none());

        cache.store(server, domain, test_config.clone(), Duration::from_secs(60));
        assert_eq!(cache.get(server, domain), Some(test_config.clone()));

        // Expired record
        cache.store(
            server,
            domain,
            test_config.clone(),
            Duration::from_millis(1),
        );
        std::thread::sleep(Duration::from_millis(10));
        assert!(cache.get(server, domain).is_none());

        // Error record
        cache.store_error(
            server,
            domain,
            "lookup timeout".into(),
            Duration::from_secs(60),
        );
        assert!(cache.get(server, domain).is_none());

        cache.clear();
        assert!(cache.get(server, domain).is_none());
    }

    #[test]
    fn test_resolve_ech_config_base64_and_fallbacks() {
        // Direct Base64
        let raw = vec![0x10, 0x20, 0x30, 0x40];
        let b64 = base64::engine::general_purpose::STANDARD.encode(&raw);
        let res = resolve_ech_config(&b64, "example.com", "full").unwrap();
        assert_eq!(res, Some(raw));

        // Missing with full force_query -> fallback dummy [1, 1, 4, 5, 1, 4]
        GLOBAL_ECH_CACHE.clear();
        let fallback = resolve_ech_config("udp://1.1.1.1", "missing.com", "full").unwrap();
        assert_eq!(fallback, Some(DUMMY_FALLBACK_ECH_CONFIG.to_vec()));

        // Missing with half/none force_query -> None
        let half = resolve_ech_config("udp://1.1.1.1", "missing.com", "half").unwrap();
        assert_eq!(half, None);

        // Pre-cached DNS entry
        let cached_cfg = vec![9, 8, 7, 6];
        GLOBAL_ECH_CACHE.store(
            "udp://1.1.1.1",
            "cached.com",
            cached_cfg.clone(),
            Duration::from_secs(60),
        );
        let hit = resolve_ech_config("udp://1.1.1.1", "cached.com", "full").unwrap();
        assert_eq!(hit, Some(cached_cfg));
    }
}
