// Module: common\buf\writer_test.rs
// 1:1 Rust unit test suite corresponding to Go common\buf\writer_test.go

#[cfg(test)]
mod tests {
    use super::super::buffer::Buffer;
    use super::super::io::Writer;
    use super::super::multi_buffer::MultiBuffer;
    use super::super::writer::{BufferedWriter, Discard, SequentialWriter};
    use std::io::Cursor;

    #[tokio::test]
    async fn test_buffered_writer() {
        let sink = Cursor::new(Vec::new());
        let mut bw = BufferedWriter::new(sink);

        let buf = Buffer::from_bytes(b"buffered data");
        bw.write_buffer(&buf).await.unwrap();
        bw.flush().await.unwrap();

        assert_eq!(bw.into_inner().into_inner(), b"buffered data");
    }

    #[tokio::test]
    async fn test_buffered_writer_write_bytes() {
        let sink = Cursor::new(Vec::new());
        let mut bw = BufferedWriter::new(sink);

        bw.write_bytes(b"streamed byte chunk").await.unwrap();
        bw.flush().await.unwrap();

        assert_eq!(bw.into_inner().into_inner(), b"streamed byte chunk");
    }

    #[tokio::test]
    async fn test_sequential_writer_and_discard() {
        let sink = Cursor::new(Vec::new());
        let mut seq = SequentialWriter::new(sink);

        let mut mb = MultiBuffer::new();
        mb.push(Buffer::from_bytes(b"hello "));
        mb.push(Buffer::from_bytes(b"world"));

        seq.write_multi_buffer(mb).await.unwrap();

        let mut discard = Discard;
        let mut mb_discard = MultiBuffer::new();
        mb_discard.push(Buffer::from_bytes(b"discarded"));
        assert!(discard.write_multi_buffer(mb_discard).await.is_ok());
    }
}
