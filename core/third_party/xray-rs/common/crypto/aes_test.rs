// Module: common\crypto\aes_test.rs
// 1:1 Rust unit test suite corresponding to Go common\crypto\aes.go

#[cfg(test)]
mod tests {
    use super::super::aes::{AesGcmCipher, new_aes_gcm};

    #[test]
    fn test_aes_gcm_128_roundtrip() {
        let key = b"0123456789abcdef"; // 16 bytes
        let nonce = b"unique_nonce"; // 12 bytes
        let plaintext = b"hello AES-128-GCM encrypted message";

        let ciphertext = AesGcmCipher::encrypt_128(key, nonce, plaintext).unwrap();
        assert_ne!(&ciphertext[..plaintext.len()], plaintext);

        let decrypted = AesGcmCipher::decrypt_128(key, nonce, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aes_gcm_256_roundtrip() {
        let key = b"0123456789abcdef0123456789abcdef"; // 32 bytes
        let nonce = b"unique_nonce"; // 12 bytes
        let plaintext = b"hello AES-256-GCM encrypted message";

        let ciphertext = AesGcmCipher::encrypt_256(key, nonce, plaintext).unwrap();
        assert_ne!(&ciphertext[..plaintext.len()], plaintext);

        let decrypted = AesGcmCipher::decrypt_256(key, nonce, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_new_aes_gcm_validation() {
        assert!(new_aes_gcm(b"0123456789abcdef").is_ok());
        assert!(new_aes_gcm(b"0123456789abcdef0123456789abcdef").is_ok());
        assert!(new_aes_gcm(b"invalid_key_len").is_err());
    }
}
