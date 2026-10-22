// Module: testing\scenarios\crypto_internal_test.rs
// Test internal stream and AEAD ciphers

#[cfg(test)]
mod tests {
    use crate::common::crypto::{AesGcmCipher, ChaCha20Cipher};

    #[test]
    fn test_chacha20_cipher_roundtrip() {
        let key = [0x42u8; 32];
        let nonce = [0x24u8; 12];
        let plaintext = b"Hello Xray-Rust ChaCha20 Stream Cipher Test Payload";

        let cipher = ChaCha20Cipher::new(&key);
        let ciphertext = cipher.encrypt(&nonce, plaintext).unwrap();
        assert_ne!(&ciphertext, plaintext);

        let decrypted = cipher.decrypt(&nonce, &ciphertext).unwrap();
        assert_eq!(&decrypted, plaintext);
    }

    #[test]
    fn test_aes_gcm_cipher_roundtrip() {
        let key = [0x13u8; 32];
        let nonce = [0x37u8; 12];
        let plaintext = b"Hello Xray-Rust AES-GCM Authenticated Encryption";

        let ciphertext = AesGcmCipher::encrypt_256(&key, &nonce, plaintext).unwrap();
        assert_ne!(&ciphertext, plaintext);

        let decrypted = AesGcmCipher::decrypt_256(&key, &nonce, &ciphertext).unwrap();
        assert_eq!(&decrypted, plaintext);
    }
}
