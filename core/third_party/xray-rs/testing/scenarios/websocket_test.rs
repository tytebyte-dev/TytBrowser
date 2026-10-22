#[cfg(test)]
mod tests {
    use crate::transport::internet::websocket::WebSocketStream;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_websocket_stream_duplex() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (raw_stream, _) = listener.accept().await.unwrap();
            let mut ws_stream = WebSocketStream::server_handshake(Box::pin(raw_stream))
                .await
                .unwrap();

            let mut buf = [0u8; 1024];
            let n = ws_stream.read(&mut buf).await.unwrap();
            ws_stream.write_all(&buf[..n]).await.unwrap();
        });

        let raw_client = tokio::net::TcpStream::connect(local_addr).await.unwrap();
        let ws_url = format!("ws://127.0.0.1:{}/ws", local_addr.port());
        let mut client_ws = WebSocketStream::client_handshake(&ws_url, None, Box::pin(raw_client))
            .await
            .unwrap();

        let msg = b"Testing WebSocket Framed Stream in Xray-Rust";
        client_ws.write_all(msg).await.unwrap();

        let mut recv = vec![0u8; msg.len()];
        client_ws.read_exact(&mut recv).await.unwrap();
        assert_eq!(&recv, msg);
    }
}
