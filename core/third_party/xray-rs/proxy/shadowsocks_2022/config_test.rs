// Module: proxy\shadowsocks_2022\config_test.rs
// 1:1 Rust unit test suite corresponding to Go proxy\shadowsocks_2022\config_test.go

#[cfg(test)]
mod tests {
    use super::super::config::Shadowsocks2022Config;
    use super::super::validator::CipherValidator;

    #[test]
    fn test_ss2022_cipher_validation() {
        let cfg = Shadowsocks2022Config {
            method: "2022-blake3-aes-128-gcm".into(),
            key: "base64key".into(),
            network: Some("tcp".into()),
        };
        assert!(CipherValidator::is_valid_method(&cfg.method));
    }
}
