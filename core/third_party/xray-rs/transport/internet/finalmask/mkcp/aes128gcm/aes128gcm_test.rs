// Module: transport\internet\finalmask\mkcp\aes128gcm\aes128gcm_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\finalmask\mkcp\aes128gcm\aes128gcm_test.go

#[cfg(test)]
mod tests {
    use super::super::conn::{Aes128GcmPacketConn, GCM_NONCE_SIZE, GCM_TAG_SIZE};
    use rand::RngCore;

    #[test]
    fn test_aes128gcm_seal_open_roundtrip() {
        let conn = Aes128GcmPacketConn::new("psk-secret");
        assert_eq!(conn.size(), GCM_NONCE_SIZE);
        assert_eq!(conn.overhead(), GCM_TAG_SIZE);

        let plaintext = b"0123456789012";
        let wrapped = conn.wrap(plaintext).expect("wrap should succeed");
        assert_eq!(
            wrapped.len(),
            GCM_NONCE_SIZE + GCM_TAG_SIZE + plaintext.len()
        );

        let opened = conn.unwrap(&wrapped).expect("unwrap should succeed");
        assert_eq!(opened, plaintext);
    }

    #[test]
    fn test_aes128gcm_bounce() {
        let conn = Aes128GcmPacketConn::new("psk-secret");
        let mut buf = vec![0u8; GCM_NONCE_SIZE + GCM_TAG_SIZE];
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            rng.fill_bytes(&mut buf);
            let result = conn.unwrap(&buf);
            assert!(
                result.is_err(),
                "Random ciphertext must fail GCM authentication"
            );
        }
    }

    #[test]
    fn test_aes128gcm_wrong_key_fails() {
        let conn1 = Aes128GcmPacketConn::new("password-1");
        let conn2 = Aes128GcmPacketConn::new("password-2");

        let plaintext = b"secret data to authenticate";
        let wrapped = conn1.wrap(plaintext).unwrap();
        let result = conn2.unwrap(&wrapped);
        assert!(result.is_err(), "Decryption with wrong password must fail");
    }
}
