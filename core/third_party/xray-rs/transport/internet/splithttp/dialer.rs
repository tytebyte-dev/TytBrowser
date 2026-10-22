// Module: transport\internet\splithttp\dialer.rs
// 1:1 Rust implementation corresponding to Go transport\internet\splithttp\dialer.go

use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use tokio::io::AsyncReadExt;

use super::client::{DefaultDialerClient, DialerClient};
use super::config::SplitHttpConfig;
use super::connection::SplitConn;
use crate::common::errors::Result;
use crate::common::net::{BoxStream, Destination};
use crate::transport::internet::dialer::Dialer;

pub struct SplitHttpDialer {
    config: SplitHttpConfig,
}

impl SplitHttpDialer {
    pub fn new(config: SplitHttpConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &SplitHttpConfig {
        &self.config
    }

    pub async fn dial_with_client(
        &self,
        client: Arc<dyn DialerClient>,
        dest: &Destination,
    ) -> Result<BoxStream> {
        let mode = if self.config.mode.is_empty() || self.config.mode == "auto" {
            "packet-up"
        } else {
            &self.config.mode
        };

        let session_id = if mode != "stream-one" {
            uuid::Uuid::new_v4().to_string()
        } else {
            String::new()
        };

        let norm_path = self.config.get_normalized_path();
        let target_url = if !self.config.host.is_empty() {
            format!("http://{}{}", self.config.host, norm_path)
        } else {
            format!("http://{}{}", dest.address, norm_path)
        };

        // Open download stream
        let (down_reader, local_addr, remote_addr) = client
            .open_stream(&target_url, &session_id, None, false)
            .await?;

        // Establish upload pipe
        let (upload_tx, mut upload_rx) = tokio::io::duplex(64 * 1024);

        let max_post_bytes = self.config.get_normalized_sc_max_each_post_bytes().rand() as usize;
        let max_chunk = if max_post_bytes > 0 {
            max_post_bytes
        } else {
            1024 * 1024
        };

        let client_clone = Arc::clone(&client);
        let session_id_clone = session_id.clone();
        let url_clone = target_url.clone();

        tokio::spawn(async move {
            let seq = AtomicI64::new(0);
            let mut buf = vec![0u8; max_chunk];
            loop {
                match upload_rx.read(&mut buf).await {
                    Ok(0) => break, // EOF, pipe closed
                    Ok(n) => {
                        let cur_seq = seq.fetch_add(1, Ordering::SeqCst);
                        let payload = buf[..n].to_vec();
                        if let Err(_) = client_clone
                            .post_packet(
                                &url_clone,
                                &session_id_clone,
                                &cur_seq.to_string(),
                                payload,
                            )
                            .await
                        {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let conn = SplitConn::new(
            down_reader,
            Box::pin(upload_tx),
            local_addr,
            remote_addr,
            None,
        );

        Ok(Box::pin(conn))
    }
}

#[async_trait]
impl Dialer for SplitHttpDialer {
    async fn dial(&self, dest: &Destination) -> Result<BoxStream> {
        let client = Arc::new(DefaultDialerClient::new(
            self.config.clone(),
            dest.clone(),
            "1.1",
        ));
        self.dial_with_client(client, dest).await
    }
}
