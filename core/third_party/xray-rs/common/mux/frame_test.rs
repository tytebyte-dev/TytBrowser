// Module: common\mux\frame_test.rs
// 1:1 Rust unit test suite corresponding to Go common\mux\frame_test.go

#[cfg(test)]
mod tests {
    use crate::common::mux::frame::{Frame, FrameMetadata, OPTION_DATA, SessionStatus};
    use crate::common::net::{Address, Destination};
    use std::io::Cursor;

    #[tokio::test]
    async fn test_frame_new_session_serialize() {
        let dest = Destination::tcp(Address::Domain("www.example.com".into()), 80);
        let frame = Frame::new_session(1, dest.clone(), b"ping".to_vec());

        let mut buf = Vec::new();
        frame.write_to(&mut buf).await.unwrap();

        let mut reader = Cursor::new(buf);
        let decoded = Frame::read_from(&mut reader).await.unwrap();

        assert_eq!(decoded.session_id, 1);
        assert_eq!(decoded.status, SessionStatus::New);
        assert_eq!(decoded.payload, b"ping");
        assert_eq!(decoded.target, Some(dest));
    }

    #[tokio::test]
    async fn test_frame_end_serialize() {
        let frame = Frame::end(42);
        let mut buf = Vec::new();
        frame.write_to(&mut buf).await.unwrap();

        let mut reader = Cursor::new(buf);
        let decoded = Frame::read_from(&mut reader).await.unwrap();

        assert_eq!(decoded.session_id, 42);
        assert_eq!(decoded.status, SessionStatus::End);
        assert!(decoded.payload.is_empty());
    }

    #[test]
    fn test_frame_metadata_roundtrip() {
        let dest = Destination::tcp(Address::Domain("www.example.com".into()), 80);
        let mut meta = FrameMetadata::new(1, SessionStatus::New);
        meta.target = Some(dest.clone());
        meta.option = OPTION_DATA;

        let mut buf = Vec::new();
        meta.write_to(&mut buf).unwrap();

        let mut decoded = FrameMetadata::new(0, SessionStatus::End);
        let read_bytes = decoded.unmarshal(&buf).unwrap();
        assert_eq!(read_bytes, buf.len());
        assert_eq!(decoded.session_id, 1);
        assert_eq!(decoded.session_status, SessionStatus::New);
        assert_eq!(decoded.option, OPTION_DATA);
        assert_eq!(decoded.target, Some(dest));
    }
}
