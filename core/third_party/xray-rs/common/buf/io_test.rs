// Module: common\buf\io_test.rs
// 1:1 Rust unit test suite corresponding to Go common\buf\io_test.go

#[cfg(test)]
mod tests {
    use super::super::io::write_all_bytes;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_write_all_bytes() {
        let mut sink = Cursor::new(Vec::new());
        let payload = b"hello all bytes";
        let n = write_all_bytes(&mut sink, payload).await.unwrap();
        assert_eq!(n, payload.len());
        assert_eq!(sink.into_inner(), payload);
    }
}
