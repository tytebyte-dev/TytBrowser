// Module: common\buf\multi_buffer_test.rs
// 1:1 Rust unit test suite corresponding to Go common\buf\multi_buffer_test.go

#[cfg(test)]
mod tests {
    use super::super::multi_buffer::MultiBuffer;

    #[test]
    fn test_multi_buffer_append_and_to_vec() {
        let mut mb1 = MultiBuffer::from_bytes(b"hello ");
        let mut mb2 = MultiBuffer::from_bytes(b"world");

        assert_eq!(mb1.len(), 6);
        assert_eq!(mb2.len(), 5);

        mb1.append(&mut mb2);
        assert_eq!(mb1.len(), 11);
        assert_eq!(mb1.to_vec(), b"hello world");

        mb1.clear();
        assert!(mb1.is_empty());
        assert_eq!(mb1.len(), 0);
    }
}
