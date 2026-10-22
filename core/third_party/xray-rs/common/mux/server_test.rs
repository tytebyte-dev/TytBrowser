// Module: common\mux\server_test.rs
// 1:1 Rust unit test suite corresponding to Go common\mux\server_test.go

#[cfg(test)]
mod tests {
    use crate::common::mux::frame::Frame;
    use crate::common::mux::server::MuxServer;
    use crate::common::net::{Address, Destination};

    #[tokio::test]
    async fn test_mux_server_init_and_dispatch() {
        let server = MuxServer::new();
        assert_eq!(server.session_count(), 0);

        let dest = Destination::tcp(Address::Domain("example.com".into()), 80);
        let new_frame = Frame::new_session(1, dest, vec![1, 2, 3]);

        let session = server.dispatch_frame(new_frame).await.unwrap();
        assert!(session.is_some());
        assert_eq!(server.session_count(), 1);

        let end_frame = Frame::end(1);
        let _ = server.dispatch_frame(end_frame).await.unwrap();
        assert_eq!(server.session_count(), 0);
    }
}
