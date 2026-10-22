// Module: common\uuid\uuid_test.rs
// 1:1 Rust unit test suite corresponding to Go common\uuid\uuid_test.go

#[cfg(test)]
mod tests {
    use super::super::uuid::*;

    #[test]
    fn test_parse_bytes() {
        let str = "2418d087-648d-4990-86e8-19dca1d006d3";
        let bytes: [u8; 16] = [
            0x24, 0x18, 0xd0, 0x87, 0x64, 0x8d, 0x49, 0x90, 0x86, 0xe8, 0x19, 0xdc, 0xa1, 0xd0,
            0x06, 0xd3,
        ];

        let u = parse_bytes(&bytes).unwrap();
        assert_eq!(u.to_string(), str);

        assert!(parse_bytes(&[1, 3, 2, 4]).is_err());
    }

    #[test]
    fn test_parse_string() {
        let str = "2418d087-648d-4990-86e8-19dca1d006d3";
        let expected_bytes: [u8; 16] = [
            0x24, 0x18, 0xd0, 0x87, 0x64, 0x8d, 0x49, 0x90, 0x86, 0xe8, 0x19, 0xdc, 0xa1, 0xd0,
            0x06, 0xd3,
        ];

        let u = parse_string(str).unwrap();
        assert_eq!(*u.bytes(), expected_bytes);

        // Deterministic short-string v5 hash matching Go
        let u0 = parse_string("example").unwrap();
        let u5 = parse_string("feb54431-301b-52bb-a6dd-e1e93e81bb9e").unwrap();
        assert_eq!(u0, u5);

        // Invalid hex character
        assert!(parse_string("2418d087-648k-4990-86e8-19dca1d006d3").is_err());

        // Invalid length
        assert!(parse_string("2418d087-648d-4990-86e8-19dca1d0").is_err());
    }

    #[test]
    fn test_new_uuid() {
        let u1 = UUID::new();
        let u2 = parse_string(&u1.to_string()).unwrap();

        assert_eq!(u1.to_string(), u2.to_string());
        assert_eq!(u1.bytes(), u2.bytes());

        // Ensure randomness
        let u3 = UUID::new();
        assert_ne!(u1.to_string(), u3.to_string());
    }

    #[test]
    fn test_equals() {
        let u1 = UUID::new();
        let u2 = u1;
        assert!(u1.equals(Some(&u2)));

        let u3 = UUID::new();
        assert!(!u1.equals(Some(&u3)));
        assert!(!u1.equals(None));
    }
}
