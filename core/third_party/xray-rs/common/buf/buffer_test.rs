// Module: common\buf\buffer_test.rs
// 1:1 Rust unit test suite corresponding to Go common\buf\buffer_test.go

#[cfg(test)]
mod tests {
    use super::super::buffer::Buffer;

    #[test]
    fn test_buffer_clear_and_empty() {
        let mut buf = Buffer::new();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);

        buf.write(b"hello world").unwrap();
        assert!(!buf.is_empty());
        assert_eq!(buf.len(), 11);
        assert_eq!(buf.as_slice(), b"hello world");

        buf.clear();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_buffer_from_bytes() {
        let buf = Buffer::from_bytes(b"xray core");
        assert_eq!(buf.len(), 9);
        assert_eq!(buf.as_slice(), b"xray core");
    }
}
