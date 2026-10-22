// Module: common\crypto\chunk_test.rs
// 1:1 Rust unit test suite corresponding to Go common\crypto\chunk_test.go

#[cfg(test)]
mod tests {
    use super::super::chunk::{AeadChaCha20ChunkReader, AeadChaCha20ChunkWriter, PlainChunk};
    use std::io::Cursor;

    #[tokio::test]
    async fn test_plain_chunk_write_read() {
        let mut buf = Vec::new();
        let payload = b"stream chunk payload";

        PlainChunk::write_chunk(&mut buf, payload).await.unwrap();

        let mut reader = Cursor::new(buf);
        let read = PlainChunk::read_chunk(&mut reader).await.unwrap();

        assert_eq!(&read, payload);
    }

    #[tokio::test]
    async fn test_aead_chacha20_chunk_roundtrip() {
        let key = [0x55u8; 32];
        let mut buf = Vec::new();
        let payload = b"authenticated encrypted chunk";

        {
            let mut writer = AeadChaCha20ChunkWriter::new(&mut buf, &key);
            writer.write_chunk(payload).await.unwrap();
        }

        let mut reader = AeadChaCha20ChunkReader::new(Cursor::new(buf), &key);
        let decrypted = reader.read_chunk().await.unwrap();

        assert_eq!(&decrypted, payload);
    }
}
