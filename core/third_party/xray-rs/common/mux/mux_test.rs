// Module: common\mux\mux_test.rs
// 1:1 Rust unit test suite corresponding to Go common\mux\mux_test.go

#[cfg(test)]
mod tests {
    use crate::common::mux::frame::{Frame, FrameType, SessionStatus};
    use crate::common::mux::reader::FrameReader;
    use crate::common::mux::writer::FrameWriter;
    use crate::common::net::{Address, Destination};
    use std::io::Cursor;

    #[test]
    fn test_mux_frame_flags() {
        let f = Frame::new(1, FrameType::Keep, vec![1, 2, 3]);
        assert_eq!(f.session_id, 1);
        assert_eq!(f.status, SessionStatus::Keep);
    }

    #[tokio::test]
    async fn test_reader_writer_roundtrip() {
        let mut stream = Vec::new();
        let mut writer = FrameWriter::new(&mut stream);

        let dest = Destination::tcp(Address::Domain("example.com".into()), 80);
        let frame1 = Frame::new_session(1, dest, b"abcd".to_vec());
        writer.write_frame(&frame1).await.unwrap();

        let frame2 = Frame::data(1, b"efgh".to_vec());
        writer.write_frame(&frame2).await.unwrap();

        let frame3 = Frame::end(1);
        writer.write_frame(&frame3).await.unwrap();

        let mut cursor = Cursor::new(stream);
        let mut reader = FrameReader::new(&mut cursor);

        let read1 = reader.read_frame().await.unwrap();
        assert_eq!(read1.session_id, 1);
        assert_eq!(read1.status, SessionStatus::New);
        assert_eq!(read1.payload, b"abcd");

        let read2 = reader.read_frame().await.unwrap();
        assert_eq!(read2.session_id, 1);
        assert_eq!(read2.status, SessionStatus::Keep);
        assert_eq!(read2.payload, b"efgh");

        let read3 = reader.read_frame().await.unwrap();
        assert_eq!(read3.session_id, 1);
        assert_eq!(read3.status, SessionStatus::End);
        assert!(read3.payload.is_empty());
    }
}
