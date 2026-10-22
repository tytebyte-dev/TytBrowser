// Module: common\\crypto\\benchmark_test.rs
// 1:1 Rust unit test suite corresponding to Go common\\crypto\\benchmark_test.go

#[cfg(test)]
mod tests {
    use super::super::chacha20::ChaCha20Cipher;

    #[test]
    fn test_crypto_keystream_speed() {
        let key = [0x42u8; 32];
        let nonce = [0x24u8; 12];
        let cipher = ChaCha20Cipher::new(&key);
        let buf = vec![0u8; 1024];
        let enc = cipher.encrypt(&nonce, &buf).unwrap();
        assert_ne!(enc, buf);
    }
}
