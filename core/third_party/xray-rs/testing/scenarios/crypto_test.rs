#[cfg(test)]
mod tests {
    use crate::common::crypto::{
        AeadChaCha20ChunkReader, AeadChaCha20ChunkWriter, IncreasingNonce, PlainChunk,
    };
    use std::io::Cursor;

    #[test]
    fn test_increasing_nonce() {
        let mut nonce_gen = IncreasingNonce::new(12);
        let n1 = nonce_gen.next().to_vec();
        let n2 = nonce_gen.next().to_vec();
        assert_eq!(n1[0], 1);
        assert_eq!(n2[0], 2);
    }

    #[tokio::test]
    async fn test_plain_chunk_roundtrip() {
        let payload = b"Hello Xray chunking";
        let mut buf = Vec::new();
        PlainChunk::write_chunk(&mut buf, payload).await.unwrap();

        let mut cursor = Cursor::new(buf);
        let read = PlainChunk::read_chunk(&mut cursor).await.unwrap();
        assert_eq!(read, payload);
    }

    #[tokio::test]
    async fn test_aead_chacha20_chunk_roundtrip() {
        let key = [0x42u8; 32];
        let mut encrypted_buf = Vec::new();

        let plaintext1 = b"Chunk number one for secure stream";
        let plaintext2 = b"Chunk number two for secure stream";

        {
            let mut writer = AeadChaCha20ChunkWriter::new(&mut encrypted_buf, &key);
            writer.write_chunk(plaintext1).await.unwrap();
            writer.write_chunk(plaintext2).await.unwrap();
        }

        let mut cursor = Cursor::new(encrypted_buf);
        let mut reader = AeadChaCha20ChunkReader::new(&mut cursor, &key);

        let dec1 = reader.read_chunk().await.unwrap();
        let dec2 = reader.read_chunk().await.unwrap();

        assert_eq!(dec1, plaintext1);
        assert_eq!(dec2, plaintext2);
    }
}
