#[path = "fragment/config.rs"]
pub mod config;
#[path = "fragment/config.pb.rs"]
pub mod config_pb;
#[path = "fragment/conn.rs"]
pub mod conn;

#[cfg(test)]
#[path = "fragment/fragment_test.rs"]
pub mod fragment_test;

pub use config::Config;
pub use conn::FragmentConn;

use crate::common::errors::Result;
use std::time::Duration;
use tokio::io::{AsyncWrite, AsyncWriteExt};

#[derive(Debug, Clone)]
pub struct FragmentConfig {
    pub min_len: usize,
    pub max_len: usize,
    pub min_delay: Duration,
    pub max_delay: Duration,
    pub packets: String,
}

impl Default for FragmentConfig {
    fn default() -> Self {
        Self {
            min_len: 50,
            max_len: 100,
            min_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(20),
            packets: "tlshello".to_string(),
        }
    }
}

pub struct Fragmenter {
    config: FragmentConfig,
}

impl Fragmenter {
    pub fn new(config: FragmentConfig) -> Self {
        Self { config }
    }

    pub fn is_tls_client_hello(buf: &[u8]) -> bool {
        // TLS Record Header: 0x16 (Handshake), 0x03, 0x01/0x03, Length, Handshake Type: 0x01 (ClientHello)
        if buf.len() > 5 && buf[0] == 0x16 && buf[1] == 0x03 && buf[5] == 0x01 {
            return true;
        }
        false
    }

    pub async fn write_fragmented<W: AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
        data: &[u8],
    ) -> Result<()> {
        if self.config.packets == "tlshello" && !Self::is_tls_client_hello(data) {
            writer.write_all(data).await?;
            return Ok(());
        }

        let mut offset = 0;
        let mut rng = rand::thread_rng();
        use rand::Rng;

        while offset < data.len() {
            let chunk_size = rng
                .gen_range(self.config.min_len..=self.config.max_len)
                .min(data.len() - offset);
            writer.write_all(&data[offset..offset + chunk_size]).await?;
            writer.flush().await?;
            offset += chunk_size;

            if offset < data.len() {
                let delay_ms = rng.gen_range(
                    self.config.min_delay.as_millis()..=self.config.max_delay.as_millis(),
                );
                tokio::time::sleep(Duration::from_millis(delay_ms as u64)).await;
            }
        }

        Ok(())
    }
}
