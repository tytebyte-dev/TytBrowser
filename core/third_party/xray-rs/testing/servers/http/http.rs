// Module: testing\servers\http\http.rs
// HTTP test server with customizable path routing and request handlers

use crate::common::errors::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::watch;

pub type HttpHandler = Arc<dyn Fn(&str, &str, &[u8]) -> (u16, Vec<u8>) + Send + Sync>;

pub struct Server {
    addr: SocketAddr,
    shutdown_tx: watch::Sender<bool>,
    requests_count: Arc<AtomicUsize>,
}

impl Server {
    pub async fn start(listen_addr: impl Into<Option<SocketAddr>>) -> Result<Self> {
        Self::start_with_handlers(listen_addr, HashMap::new()).await
    }

    pub async fn start_with_handlers(
        listen_addr: impl Into<Option<SocketAddr>>,
        handlers: HashMap<String, HttpHandler>,
    ) -> Result<Self> {
        let addr = listen_addr
            .into()
            .unwrap_or_else(|| "127.0.0.1:0".parse().unwrap());
        let listener = TcpListener::bind(addr).await?;
        let local_addr = listener.local_addr()?;

        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        let requests_count = Arc::new(AtomicUsize::new(0));
        let req_count_clone = requests_count.clone();
        let handlers = Arc::new(handlers);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    accept_res = listener.accept() => {
                        match accept_res {
                            Ok((mut stream, _)) => {
                                let req_count = req_count_clone.clone();
                                let handlers_clone = handlers.clone();

                                tokio::spawn(async move {
                                    let mut buf = [0u8; 8192];
                                    let mut total_read = 0;

                                    while total_read < buf.len() {
                                        let n = match stream.read(&mut buf[total_read..]).await {
                                            Ok(0) => break,
                                            Ok(n) => n,
                                            Err(_) => break,
                                        };
                                        total_read += n;

                                        let mut headers = [httparse::EMPTY_HEADER; 64];
                                        let mut req = httparse::Request::new(&mut headers);
                                        let parsed = req.parse(&buf[..total_read]);

                                        if let Ok(httparse::Status::Complete(header_len)) = parsed {
                                            req_count.fetch_add(1, Ordering::SeqCst);
                                            let method = req.method.unwrap_or("GET").to_string();
                                            let raw_path = req.path.unwrap_or("/");
                                            let path = if let Some(idx) = raw_path.find("://") {
                                                if let Some(slash) = raw_path[idx + 3..].find('/') {
                                                    &raw_path[idx + 3 + slash..]
                                                } else {
                                                    "/"
                                                }
                                            } else {
                                                raw_path
                                            }.to_string();

                                            let content_len = headers.iter()
                                                .find(|h| h.name.eq_ignore_ascii_case("Content-Length"))
                                                .and_then(|h| std::str::from_utf8(h.value).ok())
                                                .and_then(|s| s.parse::<usize>().ok())
                                                .unwrap_or(0);

                                            let body_start = header_len;
                                            let body_end = body_start + content_len;

                                            while total_read < body_end && total_read < buf.len() {
                                                match stream.read(&mut buf[total_read..]).await {
                                                    Ok(0) => break,
                                                    Ok(n) => total_read += n,
                                                    Err(_) => break,
                                                }
                                            }

                                            let body = if total_read >= body_end {
                                                &buf[body_start..body_end]
                                            } else {
                                                &buf[body_start..total_read]
                                            };

                                            let (status_code, resp_body) = if let Some(handler) = handlers_clone.get(&path) {
                                                handler(&method, &path, body)
                                            } else if path == "/" {
                                                (200, b"Home".to_vec())
                                            } else {
                                                (200, format!("Echo from {}", path).into_bytes())
                                            };

                                            let status_text = match status_code {
                                                200 => "OK",
                                                404 => "Not Found",
                                                500 => "Internal Server Error",
                                                _ => "OK",
                                            };

                                            let response = format!(
                                                "HTTP/1.1 {} {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                                                status_code,
                                                status_text,
                                                resp_body.len()
                                            );

                                            let _ = stream.write_all(response.as_bytes()).await;
                                            let _ = stream.write_all(&resp_body).await;
                                            let _ = stream.flush().await;
                                            break;
                                        }
                                    }
                                });
                            }
                            Err(_) => break,
                        }
                    }
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            break;
                        }
                    }
                }
            }
        });

        Ok(Self {
            addr: local_addr,
            shutdown_tx,
            requests_count,
        })
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    pub fn requests(&self) -> usize {
        self.requests_count.load(Ordering::SeqCst)
    }

    pub fn close(&self) {
        let _ = self.shutdown_tx.send(true);
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.close();
    }
}
