// Module: common\serial\serial_test.rs
// 1:1 Rust unit test suite corresponding to Go common\serial\serial_test.go

#[cfg(test)]
mod tests {
    use super::super::serial::*;
    use std::io::Cursor;

    #[test]
    fn test_uint16_serial() {
        let mut buf = Vec::new();
        let n = write_uint16(&mut buf, 10).unwrap();
        assert_eq!(n, 2);
        assert_eq!(buf, vec![0, 10]);
    }

    #[test]
    fn test_uint64_serial() {
        let mut buf = Vec::new();
        let n = write_uint64(&mut buf, 10).unwrap();
        assert_eq!(n, 8);
        assert_eq!(buf, vec![0, 0, 0, 0, 0, 0, 0, 10]);
    }

    #[test]
    fn test_read_uint16() {
        let test_cases = vec![(vec![0u8, 1], 1u16), (vec![0x12, 0x34], 0x1234u16)];

        for (input, expected) in test_cases {
            let mut reader = Cursor::new(input);
            let v = read_uint16(&mut reader).unwrap();
            assert_eq!(v, expected);
        }
    }

    #[tokio::test]
    async fn test_serial_integer_io() {
        let mut buf = Vec::new();
        write_u16(&mut buf, 0x1234).await.unwrap();
        write_u32(&mut buf, 0x12345678).await.unwrap();
        write_u64(&mut buf, 0x123456789ABCDEF0).await.unwrap();

        let mut reader = Cursor::new(buf);
        assert_eq!(read_u16(&mut reader).await.unwrap(), 0x1234);
        assert_eq!(read_u32(&mut reader).await.unwrap(), 0x12345678);
        assert_eq!(read_u64(&mut reader).await.unwrap(), 0x123456789ABCDEF0);
    }
}
