// Module: transport\internet\splithttp\hub.rs
// 1:1 Rust implementation corresponding to Go transport\internet\splithttp\hub.go

use super::config::SplitHttpConfig;
use super::connection::SplitConn;
use super::upload_queue::UploadQueue;
use crate::common::errors::{Error, Result};
use crate::common::net::BoxStream;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc};

pub struct HttpSession {
    pub upload_queue: UploadQueue,
}

impl HttpSession {
    pub fn new(max_buffered: usize) -> Self {
        Self {
            upload_queue: UploadQueue::new(max_buffered),
        }
    }
}

pub struct SplitHttpHub {
    local_addr: SocketAddr,
    config: SplitHttpConfig,
    sessions: Arc<Mutex<HashMap<String, Arc<HttpSession>>>>,
    conn_rx: Arc<Mutex<mpsc::Receiver<(BoxStream, SocketAddr)>>>,
}

impl SplitHttpHub {
    pub async fn bind(addr: SocketAddr) -> Result<Self> {
        Self::listen(addr, SplitHttpConfig::default()).await
    }

    pub async fn listen(addr: SocketAddr, config: SplitHttpConfig) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        let local_addr = listener.local_addr()?;
        let sessions = Arc::new(Mutex::new(HashMap::new()));
        let (conn_tx, conn_rx) = mpsc::channel(64);

        let sessions_clone = Arc::clone(&sessions);
        let config_clone = config.clone();

        tokio::spawn(async move {
            loop {
                let (stream, remote_addr) = match listener.accept().await {
                    Ok(res) => res,
                    Err(_) => break,
                };

                let sessions_inner = Arc::clone(&sessions_clone);
                let config_inner = config_clone.clone();
                let tx_inner = conn_tx.clone();

                tokio::spawn(async move {
                    let _ = Self::handle_connection(
                        stream,
                        remote_addr,
                        config_inner,
                        sessions_inner,
                        tx_inner,
                    )
                    .await;
                });
            }
        });

        Ok(Self {
            local_addr,
            config,
            sessions,
            conn_rx: Arc::new(Mutex::new(conn_rx)),
        })
    }

    pub fn sessions(&self) -> &Arc<Mutex<HashMap<String, Arc<HttpSession>>>> {
        &self.sessions
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.local_addr)
    }

    pub fn config(&self) -> &SplitHttpConfig {
        &self.config
    }

    pub async fn accept(&self) -> Result<(BoxStream, SocketAddr)> {
        let mut guard = self.conn_rx.lock().await;
        guard.recv().await.ok_or(Error::Closed)
    }

    async fn handle_connection(
        mut stream: TcpStream,
        remote_addr: SocketAddr,
        config: SplitHttpConfig,
        sessions: Arc<Mutex<HashMap<String, Arc<HttpSession>>>>,
        conn_tx: mpsc::Sender<(BoxStream, SocketAddr)>,
    ) -> Result<()> {
        let mut header_buf = [0u8; 4096];
        let mut n_read = 0;
        let header_end = loop {
            let n = stream.read(&mut header_buf[n_read..]).await?;
            if n == 0 {
                return Ok(());
            }
            n_read += n;
            if let Some(pos) = header_buf[..n_read]
                .windows(4)
                .position(|w| w == b"\r\n\r\n")
            {
                break pos + 4;
            }
            if n_read >= header_buf.len() {
                let _ = stream
                    .write_all(b"HTTP/1.1 431 Request Header Fields Too Large\r\n\r\n")
                    .await;
                return Ok(());
            }
        };

        let header_str = String::from_utf8_lossy(&header_buf[..header_end]);
        let mut lines = header_str.lines();
        let request_line = lines.next().unwrap_or("");
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n").await;
            return Ok(());
        }

        let method = parts[0];
        let req_uri = parts[1];

        // Parse headers
        let mut headers = HashMap::new();
        for line in lines {
            if let Some((k, v)) = line.split_once(':') {
                headers.insert(k.trim().to_lowercase(), v.trim().to_string());
            }
        }

        // Host validation
        if !config.host.is_empty()
            && let Some(host_val) = headers.get("host")
            && !host_val.eq_ignore_ascii_case(&config.host)
        {
            let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").await;
            return Ok(());
        }

        // Path validation
        let norm_path = config.get_normalized_path();
        if !req_uri.starts_with(&norm_path) {
            let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").await;
            return Ok(());
        }

        // Extract session ID and sequence
        let uri_parts: Vec<&str> = req_uri.splitn(2, '?').collect();
        let pure_path = uri_parts[0];
        let query_str = if uri_parts.len() > 1 {
            uri_parts[1]
        } else {
            ""
        };

        let mut query_map = HashMap::new();
        for pair in query_str.split('&').filter(|s| !s.is_empty()) {
            if let Some((k, v)) = pair.split_once('=') {
                query_map.insert(k.to_string(), v.to_string());
            }
        }

        let mut cookie_map = HashMap::new();
        if let Some(cookie_hdr) = headers.get("cookie") {
            for c in cookie_hdr.split(';').filter(|s| !s.is_empty()) {
                if let Some((k, v)) = c.split_once('=') {
                    cookie_map.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }

        let (mut session_id, mut seq_str) = config.extract_meta_from_request(
            pure_path,
            &norm_path,
            &headers,
            &query_map,
            &cookie_map,
        );

        if session_id.is_empty()
            && let Some(sid) = headers.get("x-session-id")
        {
            session_id = sid.clone();
        }
        if seq_str.is_empty()
            && let Some(sq) = headers.get("x-seq-id")
        {
            seq_str = sq.clone();
        }

        if method == "POST" || !seq_str.is_empty() {
            // Uplink packet handling
            let content_len: usize = headers
                .get("content-length")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);

            let mut body_bytes = Vec::with_capacity(content_len);
            let initial_body = &header_buf[header_end..n_read];
            body_bytes.extend_from_slice(initial_body);

            while body_bytes.len() < content_len {
                let to_read = (content_len - body_bytes.len()).min(4096);
                let mut chunk = vec![0u8; to_read];
                let n = stream.read(&mut chunk).await?;
                if n == 0 {
                    break;
                }
                body_bytes.extend_from_slice(&chunk[..n]);
            }

            let seq = seq_str.parse::<u64>().unwrap_or(0);

            let session = {
                let mut map = sessions.lock().await;
                map.entry(session_id.clone())
                    .or_insert_with(|| {
                        Arc::new(HttpSession::new(
                            config.get_normalized_sc_max_buffered_posts(),
                        ))
                    })
                    .clone()
            };

            let _ = session.upload_queue.push(body_bytes, seq).await;

            // Respond 200 OK
            let resp = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\nCache-Control: no-store\r\n\r\n";
            stream.write_all(resp.as_bytes()).await?;
            stream.flush().await?;
            return Ok(());
        }

        if method == "GET" {
            // Downlink stream handling
            let session = {
                let mut map = sessions.lock().await;
                map.entry(session_id.clone())
                    .or_insert_with(|| {
                        Arc::new(HttpSession::new(
                            config.get_normalized_sc_max_buffered_posts(),
                        ))
                    })
                    .clone()
            };

            // Write 200 OK HTTP stream headers
            let resp_headers = "HTTP/1.1 200 OK\r\n\
                Content-Type: application/octet-stream\r\n\
                X-Accel-Buffering: no\r\n\
                Cache-Control: no-store\r\n\r\n";
            stream.write_all(resp_headers.as_bytes()).await?;
            stream.flush().await?;

            let (down_tx, mut down_rx) = tokio::io::duplex(64 * 1024);

            let (tcp_read, mut tcp_write) = stream.into_split();
            let _ = tcp_read; // Keep reader alive

            // Background task: take data written to down_tx and send to client
            tokio::spawn(async move {
                let mut buf = [0u8; 8192];
                loop {
                    match down_rx.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            if tcp_write.write_all(&buf[..n]).await.is_err() {
                                break;
                            }
                            if tcp_write.flush().await.is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            });

            // For reading client upload data from upload_queue
            let (q_tx, q_rx) = tokio::io::duplex(64 * 1024);
            let uq = session.upload_queue.clone();

            tokio::spawn(async move {
                let mut writer = q_tx;
                loop {
                    match uq.pop().await {
                        Some(data) => {
                            if writer.write_all(&data).await.is_err() {
                                break;
                            }
                            if writer.flush().await.is_err() {
                                break;
                            }
                        }
                        None => break,
                    }
                }
            });

            let conn = SplitConn::new(
                Box::pin(q_rx),
                Box::pin(down_tx),
                Some(remote_addr),
                Some(remote_addr),
                None,
            );

            let _ = conn_tx.send((Box::pin(conn), remote_addr)).await;
        }

        Ok(())
    }
}

pub use SplitHttpHub as Listener;
