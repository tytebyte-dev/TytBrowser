// Module: proxy\shadowsocks\protocol_test.rs
// 1:1 Rust unit test suite corresponding to Go proxy\shadowsocks\protocol_test.go

#[cfg(test)]
mod tests {
    use super::super::shadowsocks::*;

    #[test]
    fn test_shadowsocks_cipher_constants() {
        assert_eq!(CIPHER_AES_128_GCM, "aes-128-gcm");
        assert_eq!(CIPHER_AES_256_GCM, "aes-256-gcm");
        assert_eq!(CIPHER_CHACHA20_POLY1305, "chacha20-poly1305");
    }
}
