// Module: common\buf\reader_test.rs
// 1:1 Rust unit test suite corresponding to Go common\buf\reader_test.go

#[cfg(test)]
mod tests {
    use super::super::io::Reader;
    use super::super::reader::{BufferedReader, PacketReader, SingleReader, read_buffer};
    use std::io::Cursor;

    #[tokio::test]
    async fn test_buffered_reader() {
        let data = Cursor::new(b"abcdef".to_vec());
        let mut br = BufferedReader::new(data, 1024);

        let buf = br.read_buffer().await.unwrap();
        assert!(buf.is_some());
        assert_eq!(buf.unwrap().as_slice(), b"abcdef");

        let eof = br.read_buffer().await.unwrap();
        assert!(eof.is_none());
    }

    #[tokio::test]
    async fn test_buffered_reader_read_at_most() {
        let data = Cursor::new(b"1234567890".to_vec());
        let mut br = BufferedReader::new(data, 1024);

        let chunk1 = br.read_at_most(4).await.unwrap();
        assert_eq!(chunk1.len(), 4);
        assert_eq!(chunk1.to_vec(), b"1234");

        let chunk2 = br.read_at_most(10).await.unwrap();
        assert_eq!(chunk2.len(), 6);
        assert_eq!(chunk2.to_vec(), b"567890");
    }

    #[tokio::test]
    async fn test_single_and_packet_reader() {
        let mut cursor1 = Cursor::new(b"single data".to_vec());
        let mut single_r = SingleReader::new(&mut cursor1);
        let mb1 = single_r.read_multi_buffer().await.unwrap();
        assert_eq!(mb1.to_vec(), b"single data");

        let mut cursor2 = Cursor::new(b"packet data".to_vec());
        let mut packet_r = PacketReader::new(&mut cursor2);
        let mb2 = packet_r.read_multi_buffer().await.unwrap();
        assert_eq!(mb2.to_vec(), b"packet data");

        let mut cursor3 = Cursor::new(b"direct buffer".to_vec());
        let b = read_buffer(&mut cursor3).await.unwrap();
        assert_eq!(b.as_slice(), b"direct buffer");
    }
}
