// Module: proxy\vmess\aead\encrypt_test.rs
// 1:1 Rust unit test suite corresponding to Go proxy\vmess\aead\encrypt_test.go

#[cfg(test)]
mod tests {
    use super::super::encrypt::VmessAeadEncryptor;

    #[test]
    fn test_vmess_aead_encrypt_decrypt() {
        let key = [0x42u8; 16];
        let nonce = [0x24u8; 12];
        let enc = VmessAeadEncryptor::new(&key);
        let msg = b"vmess encrypted header";
        let ct = enc.seal(&nonce, msg).unwrap();
        assert_ne!(&ct, msg);
        let pt = enc.open(&nonce, &ct).unwrap();
        assert_eq!(&pt, msg);
    }
}
