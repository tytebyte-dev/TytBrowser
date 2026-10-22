// Module: transport\internet\kcp\connection_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\kcp\connection_test.go

#[cfg(test)]
mod tests {
    use super::super::connection::KcpConnection;
    use std::sync::Arc;
    use tokio::net::UdpSocket;

    #[tokio::test]
    async fn test_kcp_connection_lifecycle() {
        let sock1 = Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap());
        let addr1 = sock1.local_addr().unwrap();

        let sock2 = Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap());
        let addr2 = sock2.local_addr().unwrap();

        let client = KcpConnection::with_remote_addr(1024, sock1.clone(), Some(addr2));
        let server = KcpConnection::with_remote_addr(1024, sock2.clone(), Some(addr1));

        assert_eq!(client.conv(), 1024);
        assert_eq!(server.conv(), 1024);

        let test_payload = b"Hello robust KCP ARQ transmission in Rust!";
        client.send_data(test_payload).await.unwrap();

        // Server receives UDP packet from sock2 and inputs into KCP engine
        let mut raw = vec![0u8; 1500];
        let (n_raw, _src) = sock2.recv_from(&mut raw).await.unwrap();
        server.input_packet(&raw[..n_raw]).await.unwrap();

        // Read reassembled data from server
        let mut read_buf = vec![0u8; 1024];
        let n = server.recv(&mut read_buf).await.unwrap();
        assert_eq!(&read_buf[..n], test_payload);

        // Server sends ACK back to client, client processes ACK
        let (n_ack, _src) = sock1.recv_from(&mut raw).await.unwrap();
        client.input_packet(&raw[..n_ack]).await.unwrap();

        // Send window should be acknowledged and cleared
        let s_wnd = client.send_window.lock().await;
        assert_eq!(s_wnd.len(), 0);
    }
}
