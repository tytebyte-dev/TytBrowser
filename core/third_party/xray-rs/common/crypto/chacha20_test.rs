// Module: common\crypto\chacha20_test.rs
// 1:1 Rust unit test suite corresponding to Go common\crypto\chacha20_test.go

#[cfg(test)]
mod tests {
    use super::super::chacha20::ChaCha20Cipher;

    #[test]
    fn test_chacha20_encrypt_decrypt_roundtrip() {
        let key = [0x42u8; 32];
        let nonce = [0x24u8; 12];
        let cipher = ChaCha20Cipher::new(&key);

        let plaintext = b"Hello, secure Chacha20 encryption!";
        let ciphertext = cipher.encrypt(&nonce, plaintext).unwrap();

        assert_ne!(&ciphertext, plaintext);

        let decrypted = cipher.decrypt(&nonce, &ciphertext).unwrap();
        assert_eq!(&decrypted, plaintext);
    }
}
