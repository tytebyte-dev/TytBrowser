// Module: transport\internet\finalmask\mkcp\original\simple_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\finalmask\mkcp\original\simple_test.go

#[cfg(test)]
mod tests {
    use super::super::conn::SimpleAead;
    use rand::RngCore;

    #[test]
    fn test_simple_seal_open_roundtrip() {
        let aead = SimpleAead::new();
        assert_eq!(aead.nonce_size(), 0);
        assert_eq!(aead.overhead(), 6);

        let text = b"0123456789012";
        let sealed = aead.seal(text);
        assert_eq!(sealed.len(), aead.overhead() + text.len());

        let opened = aead.open(&sealed).expect("open should succeed");
        assert_eq!(opened, text);
    }

    #[test]
    fn test_simple_bounce() {
        let aead = SimpleAead::new();
        let mut buf = vec![0u8; aead.overhead() + 16];
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            rng.fill_bytes(&mut buf);
            let result = aead.open(&buf);
            assert!(result.is_err(), "Random bytes must fail authentication");
        }
    }

    #[test]
    fn test_simple_various_lengths() {
        let aead = SimpleAead::new();
        for len in 0..128 {
            let mut data = vec![0u8; len];
            rand::thread_rng().fill_bytes(&mut data);
            let sealed = aead.seal(&data);
            let opened = aead.open(&sealed).unwrap();
            assert_eq!(opened, data);
        }
    }
}
