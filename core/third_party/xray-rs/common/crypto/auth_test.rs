// Module: common\crypto\auth_test.rs
// 1:1 Rust unit test suite corresponding to Go common\crypto\auth_test.go

#[cfg(test)]
mod tests {
    use super::super::auth::Authentication;

    #[test]
    fn test_hmac_sha256_verify() {
        let key = b"secret-test-key";
        let data = b"payload to authenticate";

        let mac = Authentication::hmac_sha256(key, data).unwrap();
        assert_eq!(mac.len(), 32);

        assert!(Authentication::verify_hmac_sha256(key, data, &mac));
        assert!(!Authentication::verify_hmac_sha256(
            b"wrong-key",
            data,
            &mac
        ));
        assert!(!Authentication::verify_hmac_sha256(
            key,
            b"corrupted data",
            &mac
        ));
    }
}
