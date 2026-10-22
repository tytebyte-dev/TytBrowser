// Module: app\observatory\burst\ping.rs
// 1:1 Rust implementation corresponding to Go app\observatory\burst\ping.go

use super::burst::RTT_FAILED;
use crate::common::errors::{Error, Result};
use std::time::{Duration, Instant};

pub struct PingClient {
    pub destination: String,
    pub timeout: Duration,
}

impl PingClient {
    pub fn new(destination: impl Into<String>, timeout: Duration) -> Self {
        Self {
            destination: destination.into(),
            timeout,
        }
    }

    pub async fn measure_delay(&self, _http_method: &str) -> (Duration, Result<()>) {
        if self.destination.is_empty() {
            return (RTT_FAILED, Err(Error::Config("empty destination".into())));
        }

        let start = Instant::now();
        let host = if let Some(stripped) = self.destination.strip_prefix("https://") {
            stripped.split('/').next().unwrap_or(stripped)
        } else if let Some(stripped) = self.destination.strip_prefix("http://") {
            stripped.split('/').next().unwrap_or(stripped)
        } else {
            &self.destination
        };

        let target_addr = if host.contains(':') {
            host.to_string()
        } else if self.destination.starts_with("https://") {
            format!("{}:443", host)
        } else {
            format!("{}:80", host)
        };

        let conn_res =
            tokio::time::timeout(self.timeout, tokio::net::TcpStream::connect(&target_addr)).await;
        match conn_res {
            Ok(Ok(_stream)) => (start.elapsed(), Ok(())),
            Ok(Err(e)) => (RTT_FAILED, Err(Error::Io(e))),
            Err(_) => (RTT_FAILED, Err(Error::Timeout)),
        }
    }
}
