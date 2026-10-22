// Module: proxy\vmess\aead\authid_test.rs
// 1:1 Rust unit test suite corresponding to Go proxy\vmess\aead\authid_test.go

#[cfg(test)]
mod tests {
    use super::super::authid::AuthIdGenerator;

    #[test]
    fn test_auth_id_generation_and_matching() {
        let generator = AuthIdGenerator::new(b"secret_key_12345");
        let now = 1700000000;
        let id = generator.create_auth_id(now);
        assert!(generator.matches(&id, now + 10, 30));
        assert!(!generator.matches(&id, now + 100, 30));
    }
}
