#[cfg(test)]
mod tests {
    use crate::common::buf::{Buffer, MultiBuffer};

    #[test]
    fn test_buffer_read_write_advance() {
        let mut buf = Buffer::new();
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.capacity(), 8192);

        let data = b"Hello Xray Buffer World!";
        let n = buf.write(data).unwrap();
        assert_eq!(n, data.len());
        assert_eq!(buf.len(), data.len());
        assert_eq!(buf.as_slice(), data);

        let mut out = [0u8; 5];
        let read_n = buf.read(&mut out);
        assert_eq!(read_n, 5);
        assert_eq!(&out, b"Hello");
        assert_eq!(buf.as_slice(), b" Xray Buffer World!");

        buf.advance(6);
        assert_eq!(buf.as_slice(), b"Buffer World!");

        buf.clear();
        assert!(buf.is_empty());
    }

    #[test]
    fn test_multi_buffer_chunking_and_merge() {
        let mut mb = MultiBuffer::new();
        let chunk1 = vec![0xaa; 10000]; // Larger than single buffer 8192
        mb.append_bytes(&chunk1);
        assert_eq!(mb.len(), 10000);

        let chunk2 = vec![0xbb; 5000];
        mb.append_bytes(&chunk2);
        assert_eq!(mb.len(), 15000);

        let mut read_dest = vec![0u8; 15000];
        let total = mb.read_bytes(&mut read_dest);
        assert_eq!(total, 15000);
        assert_eq!(&read_dest[..10000], &chunk1[..]);
        assert_eq!(&read_dest[10000..], &chunk2[..]);
        assert!(mb.is_empty());
    }
}
