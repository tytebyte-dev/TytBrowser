// Module: common\buf\readv_test.rs
// 1:1 Rust unit test suite corresponding to Go common\buf\readv_test.go

#[cfg(test)]
mod tests {
    use super::super::io::Reader;
    use super::super::readv_reader::{AllocStrategy, ReadVReader};
    use std::io::Cursor;

    #[test]
    fn test_alloc_strategy() {
        let mut strategy = AllocStrategy::new();
        assert_eq!(strategy.current(), 1);

        // Adjust up (doubling)
        strategy.adjust(1);
        assert_eq!(strategy.current(), 2);

        strategy.adjust(2);
        assert_eq!(strategy.current(), 4);

        strategy.adjust(4);
        assert_eq!(strategy.current(), 8);

        // Cap at 8
        strategy.adjust(8);
        assert_eq!(strategy.current(), 8);

        // Adjust down
        strategy.adjust(3);
        assert_eq!(strategy.current(), 3);

        let bufs = strategy.alloc();
        assert_eq!(bufs.len(), 3);
    }

    #[tokio::test]
    async fn test_readv_reader_stream() {
        let data = b"readv multi buffer test data";
        let cursor = Cursor::new(data.to_vec());
        let mut reader = ReadVReader::new(cursor);
        let mb = reader.read_multi(128).await.unwrap();
        assert_eq!(mb.len(), data.len());
    }

    #[tokio::test]
    async fn test_readv_reader_trait() {
        let data = b"multi-buffer chunk payload";
        let cursor = Cursor::new(data.to_vec());
        let mut reader = ReadVReader::new(cursor);
        let mb = reader.read_multi_buffer().await.unwrap();
        assert_eq!(mb.len(), data.len());
    }
}
