// Module: common\mux\client_test.rs
// 1:1 Rust unit test suite corresponding to Go common\mux\client_test.go

#[cfg(test)]
mod tests {
    use crate::common::mux::client::MuxClient;
    use crate::common::net::{Address, Destination};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_mux_client_status_and_lifecycle() {
        let client = MuxClient::new();
        assert_eq!(client.active_sessions(), 0);

        let dest = Destination::tcp(Address::Domain("example.com".into()), 443);
        let (tx, _rx) = mpsc::channel(16);
        let (session, frame) = client.new_session(dest.clone(), tx).await.unwrap();

        assert_eq!(client.active_sessions(), 1);
        assert_eq!(session.id, 1);
        assert_eq!(frame.session_id, 1);

        let end_frame = client.close_session(session.id).await;
        assert!(end_frame.is_some());
        assert_eq!(client.active_sessions(), 0);
    }
}
