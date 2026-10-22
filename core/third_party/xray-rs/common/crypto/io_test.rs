// Module: common\crypto\io_test.rs
// 1:1 Rust unit test suite corresponding to Go common\crypto\io.go and crypto.go

#[cfg(test)]
mod tests {
    use super::super::crypto::{StreamCipher, rand_between, rand_bytes_between};
    use super::super::internal::chacha::ChaChaCore;
    use super::super::io::{CryptionReader, CryptionWriter, xor_buffers};
    use std::io::Cursor;

    #[test]
    fn test_rand_between_and_bytes() {
        for _ in 0..100 {
            let val = rand_between(10, 20);
            assert!(val >= 10 && val < 20);
        }

        let mut bytes = [0u8; 64];
        rand_bytes_between(&mut bytes, b'a', b'z');
        for b in bytes {
            assert!(b >= b'a' && b <= b'z');
        }
    }

    #[test]
    fn test_xor_buffers() {
        let mut dst = [0u8; 4];
        let src = [1, 2, 3, 4];
        let key = [0xff, 0x00];
        let len = xor_buffers(&mut dst, &src, &key).unwrap();
        assert_eq!(len, 4);
        assert_eq!(dst, [1 ^ 0xff, 2 ^ 0x00, 3 ^ 0xff, 4 ^ 0x00]);
    }

    struct SimpleXorCipher {
        key: u8,
    }

    impl StreamCipher for SimpleXorCipher {
        fn encrypt(&mut self, buffer: &mut [u8]) -> crate::common::errors::Result<()> {
            for b in buffer.iter_mut() {
                *b ^= self.key;
            }
            Ok(())
        }

        fn decrypt(&mut self, buffer: &mut [u8]) -> crate::common::errors::Result<()> {
            self.encrypt(buffer)
        }
    }

    #[tokio::test]
    async fn test_cryption_reader_and_writer() {
        let plaintext = b"cryption stream test payload";
        let mut sink = Cursor::new(Vec::new());

        let mut writer = CryptionWriter::new(&mut sink, SimpleXorCipher { key: 0x5a });
        let mut data = plaintext.to_vec();
        writer.write(&mut data).await.unwrap();

        let encrypted = sink.into_inner();
        assert_ne!(encrypted, plaintext);

        let source = Cursor::new(encrypted);
        let mut reader = CryptionReader::new(source, SimpleXorCipher { key: 0x5a });
        let mut decrypted = vec![0u8; plaintext.len()];
        let n = reader.read(&mut decrypted).await.unwrap();
        assert_eq!(n, plaintext.len());
        assert_eq!(&decrypted[..n], plaintext);
    }

    #[test]
    fn test_chacha_core_keystream() {
        let key = [0x42u8; 32];
        let nonce = [0x24u8; 12];
        let mut core = ChaChaCore::new(&key, &nonce);

        let mut block1 = [0u8; 64];
        core.block(&mut block1);
        assert_ne!(block1, [0u8; 64]);

        let mut block2 = [0u8; 64];
        core.block(&mut block2);
        assert_ne!(block2, [0u8; 64]);
        assert_ne!(block1, block2); // counter incremented!
    }
}
