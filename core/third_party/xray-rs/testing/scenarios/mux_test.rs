#[cfg(test)]
mod tests {
    use crate::common::mux::{Frame, SessionStatus};
    use crate::common::net::{Address, Destination};
    use std::io::Cursor;

    #[tokio::test]
    async fn test_mux_new_session_frame_roundtrip() {
        let dest = Destination::tcp(Address::Domain("api.openai.com".into()), 443);
        let payload = b"GET /v1/models HTTP/1.1\r\n\r\n";
        let frame = Frame::new_session(1001, dest.clone(), payload.to_vec());

        let mut buf = Vec::new();
        frame.encode(&mut buf).await.unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded = Frame::decode(&mut cursor).await.unwrap();

        assert_eq!(decoded.session_id, 1001);
        assert_eq!(decoded.status, SessionStatus::New);
        assert_eq!(decoded.target, Some(dest));
        assert_eq!(decoded.payload, payload);
    }

    #[tokio::test]
    async fn test_mux_data_and_end_frames() {
        let data_payload = b"Streaming response chunk 123";
        let data_frame = Frame::data(1001, data_payload.to_vec());

        let mut buf = Vec::new();
        data_frame.encode(&mut buf).await.unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded = Frame::decode(&mut cursor).await.unwrap();

        assert_eq!(decoded.session_id, 1001);
        assert_eq!(decoded.status, SessionStatus::Keep);
        assert_eq!(decoded.target, None);
        assert_eq!(decoded.payload, data_payload);

        let end_frame = Frame::end(1001);
        let mut end_buf = Vec::new();
        end_frame.encode(&mut end_buf).await.unwrap();

        let mut end_cursor = Cursor::new(end_buf);
        let decoded_end = Frame::decode(&mut end_cursor).await.unwrap();
        assert_eq!(decoded_end.session_id, 1001);
        assert_eq!(decoded_end.status, SessionStatus::End);
        assert!(decoded_end.payload.is_empty());
    }
}
