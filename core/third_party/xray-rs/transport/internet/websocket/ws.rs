// Module: transport\internet\websocket\ws.rs
// 1:1 Rust implementation corresponding to Go transport\internet\websocket\ws.go

use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{accept_async, client_async};

use crate::common::errors::{Error, Result};
use crate::common::net::BoxStream;

pub struct WebSocketStream;

impl WebSocketStream {
    pub async fn client_handshake(
        url: &str,
        host_header: Option<&str>,
        stream: BoxStream,
    ) -> Result<BoxStream> {
        let mut req = url
            .into_client_request()
            .map_err(|e| Error::Protocol(format!("Invalid WebSocket URL '{}': {}", url, e)))?;

        if let Some(host) = host_header
            && let Ok(val) = host.parse()
        {
            req.headers_mut()
                .insert(tokio_tungstenite::tungstenite::http::header::HOST, val);
        }

        let (ws_stream, _) = client_async(req, stream)
            .await
            .map_err(|e| Error::Protocol(format!("WebSocket client handshake failed: {}", e)))?;

        let (mut ws_sink, mut ws_stream) = ws_stream.split();
        let (mut r_client, w_server) = tokio::io::duplex(64 * 1024);
        let (r_server, mut w_client) = tokio::io::duplex(64 * 1024);

        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            use tokio::io::AsyncReadExt;
            while let Ok(n) = r_client.read(&mut buf).await {
                if n == 0 {
                    break;
                }
                if ws_sink
                    .send(Message::Binary(buf[..n].to_vec()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });

        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            while let Some(Ok(msg)) = ws_stream.next().await {
                match msg {
                    Message::Binary(bin) => {
                        if w_client.write_all(&bin).await.is_err() {
                            break;
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        });

        let combined = CombinedStream {
            reader: r_server,
            writer: w_server,
        };
        Ok(Box::pin(combined))
    }

    pub async fn server_handshake(stream: BoxStream) -> Result<BoxStream> {
        let ws_stream = accept_async(stream)
            .await
            .map_err(|e| Error::Protocol(format!("WebSocket server accept failed: {}", e)))?;

        let (mut ws_sink, mut ws_stream) = ws_stream.split();
        let (mut r_client, w_server) = tokio::io::duplex(64 * 1024);
        let (r_server, mut w_client) = tokio::io::duplex(64 * 1024);

        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            use tokio::io::AsyncReadExt;
            while let Ok(n) = r_client.read(&mut buf).await {
                if n == 0 {
                    break;
                }
                if ws_sink
                    .send(Message::Binary(buf[..n].to_vec()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });

        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            while let Some(Ok(msg)) = ws_stream.next().await {
                match msg {
                    Message::Binary(bin) => {
                        if w_client.write_all(&bin).await.is_err() {
                            break;
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        });

        let combined = CombinedStream {
            reader: r_server,
            writer: w_server,
        };
        Ok(Box::pin(combined))
    }
}

pub struct CombinedStream<R, W> {
    pub reader: R,
    pub writer: W,
}

impl<R: tokio::io::AsyncRead + Unpin, W: Unpin> tokio::io::AsyncRead for CombinedStream<R, W> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.reader).poll_read(cx, buf)
    }
}

impl<R: Unpin, W: tokio::io::AsyncWrite + Unpin> tokio::io::AsyncWrite for CombinedStream<R, W> {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::pin::Pin::new(&mut self.writer).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.writer).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.writer).poll_shutdown(cx)
    }
}
