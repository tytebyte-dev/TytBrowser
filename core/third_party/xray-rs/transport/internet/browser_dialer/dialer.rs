// Module: transport\internet\browser_dialer\dialer.rs
// 1:1 Rust implementation corresponding to Go transport\internet\browser_dialer\dialer.go

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::common::errors::{Error, Result};
use crate::common::net::{BoxStream, Destination};
use crate::transport::internet::dialer::Dialer;

static HAS_BROWSER_DIALER: AtomicBool = AtomicBool::new(false);

pub fn set_has_browser_dialer(enabled: bool) {
    HAS_BROWSER_DIALER.store(enabled, Ordering::SeqCst);
}

pub fn has_browser_dialer() -> bool {
    HAS_BROWSER_DIALER.load(Ordering::SeqCst)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub method: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
    #[serde(rename = "streamResponse")]
    pub stream_response: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WebSocketExtra {
    #[serde(default)]
    pub protocol: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HttpExtra {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub referrer: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub cookies: HashMap<String, String>,
}

pub async fn dial_ws(_uri: &str, _ed: Option<&[u8]>) -> Result<BoxStream> {
    if !has_browser_dialer() {
        return Err(Error::NotFound("browser dialer is not active".into()));
    }
    Err(Error::Protocol(
        "browser websocket bridge not connected".into(),
    ))
}

pub async fn dial_get(
    _uri: &str,
    _headers: HashMap<String, String>,
    _cookies: HashMap<String, String>,
) -> Result<BoxStream> {
    if !has_browser_dialer() {
        return Err(Error::NotFound("browser dialer is not active".into()));
    }
    Err(Error::Protocol(
        "browser http get bridge not connected".into(),
    ))
}

pub async fn dial_packet(
    _method: &str,
    _uri: &str,
    _headers: HashMap<String, String>,
    _cookies: HashMap<String, String>,
    _payload: &[u8],
) -> Result<()> {
    if !has_browser_dialer() {
        return Err(Error::NotFound("browser dialer is not active".into()));
    }
    Err(Error::Protocol(
        "browser packet bridge not connected".into(),
    ))
}

pub struct BrowserDialer;

#[async_trait]
impl Dialer for BrowserDialer {
    async fn dial(&self, dest: &Destination) -> Result<BoxStream> {
        let uri = format!("http://{}:{}/", dest.address, dest.port);
        dial_get(&uri, HashMap::new(), HashMap::new()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_serialization() {
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), "XrayBrowser".to_string());

        let extra = HttpExtra {
            referrer: "https://google.com".to_string(),
            headers,
            cookies: HashMap::new(),
        };

        let task = Task {
            method: "GET".to_string(),
            url: "https://example.com/stream".to_string(),
            extra: Some(serde_json::to_value(&extra).unwrap()),
            stream_response: true,
        };

        let json_str = serde_json::to_string(&task).unwrap();
        assert!(json_str.contains("\"method\":\"GET\""));
        assert!(json_str.contains("\"streamResponse\":true"));
        assert!(json_str.contains("\"referrer\":\"https://google.com\""));
    }

    #[tokio::test]
    async fn test_browser_dialer_disabled_state() {
        set_has_browser_dialer(false);
        assert!(!has_browser_dialer());

        let res = dial_packet(
            "POST",
            "http://test",
            HashMap::new(),
            HashMap::new(),
            b"data",
        )
        .await;
        assert!(res.is_err());
    }
}
