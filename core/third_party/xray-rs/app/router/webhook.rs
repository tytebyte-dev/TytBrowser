// Module: app\router\webhook.rs
// 1:1 Rust implementation corresponding to Go app\router\webhook.go

use crate::common::errors::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub fn parse_url(raw: &str) -> (String, String) {
    if raw.is_empty() || (!raw.starts_with('/') && !raw.starts_with('\\') && !raw.starts_with('@'))
    {
        return (raw.to_string(), String::new());
    }
    if let Some(idx) = raw.find(":/") {
        let http_url = format!("http://localhost{}", &raw[idx + 1..]);
        let socket_path = raw[..idx].to_string();
        (http_url, socket_path)
    } else {
        ("http://localhost/".to_string(), raw.to_string())
    }
}

pub fn resolve_socket_path(path: &str) -> String {
    if path.is_empty() || !path.starts_with('@') {
        return path.to_string();
    }
    // Abstract unix socket path handling
    if path.starts_with("@@") {
        path[1..].to_string()
    } else {
        path.to_string()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookConfig {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub deduplication: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
    #[serde(rename = "originalTarget", skip_serializing_if = "Option::is_none")]
    pub original_target: Option<String>,
    #[serde(rename = "routeTarget", skip_serializing_if = "Option::is_none")]
    pub route_target: Option<String>,
    #[serde(rename = "inboundTag", skip_serializing_if = "Option::is_none")]
    pub inbound_tag: Option<String>,
    #[serde(rename = "inboundName", skip_serializing_if = "Option::is_none")]
    pub inbound_name: Option<String>,
    #[serde(rename = "inboundLocal", skip_serializing_if = "Option::is_none")]
    pub inbound_local: Option<String>,
    #[serde(rename = "outboundTag", skip_serializing_if = "Option::is_none")]
    pub outbound_tag: Option<String>,
    pub ts: i64,
}

impl WebhookEvent {
    pub fn new(outbound_tag: impl Into<String>) -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        Self {
            email: None,
            level: None,
            protocol: None,
            network: None,
            source: None,
            destination: None,
            original_target: None,
            route_target: None,
            inbound_tag: None,
            inbound_name: None,
            inbound_local: None,
            outbound_tag: Some(outbound_tag.into()),
            ts,
        }
    }
}

pub struct WebhookNotifier {
    pub url: String,
    pub socket_path: String,
    pub headers: HashMap<String, String>,
    pub deduplication: u32,
    seen: Arc<RwLock<HashMap<String, Instant>>>,
    closed: Arc<AtomicBool>,
}

impl WebhookNotifier {
    pub fn new(cfg: &WebhookConfig) -> Result<Option<Self>> {
        if cfg.url.is_empty() {
            return Ok(None);
        }
        let (http_url, socket_path) = parse_url(&cfg.url);
        let resolved_socket = resolve_socket_path(&socket_path);

        Ok(Some(Self {
            url: http_url,
            socket_path: resolved_socket,
            headers: cfg.headers.clone(),
            deduplication: cfg.deduplication,
            seen: Arc::new(RwLock::new(HashMap::new())),
            closed: Arc::new(AtomicBool::new(false)),
        }))
    }

    pub fn is_duplicate(&self, email: &str) -> bool {
        if self.deduplication == 0 || email.is_empty() {
            return false;
        }
        let ttl = Duration::from_secs(self.deduplication as u64);
        let now = Instant::now();

        if let Ok(mut seen) = self.seen.write() {
            if let Some(&prev) = seen.get(email)
                && now.duration_since(prev) < ttl
            {
                return true;
            }
            seen.insert(email.to_string(), now);
        }
        false
    }

    pub fn build_event(
        outbound_tag: &str,
        inbound_tag: Option<&str>,
        email: Option<&str>,
        network: Option<&str>,
        source: Option<&str>,
        destination: Option<&str>,
    ) -> WebhookEvent {
        let mut ev = WebhookEvent::new(outbound_tag);
        ev.inbound_tag = inbound_tag.map(|s| s.to_string());
        ev.email = email.map(|s| s.to_string());
        ev.network = network.map(|s| s.to_string());
        ev.source = source.map(|s| s.to_string());
        ev.destination = destination.map(|s| s.to_string());
        ev
    }

    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }
}

// Backward-compatible types for scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub inbound_tag: String,
    pub outbound_tag: String,
    pub destination: String,
    pub source: Option<String>,
}

pub struct WebhookSender {
    url: String,
}

impl WebhookSender {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn format_payload(&self, payload: &WebhookPayload) -> Result<String> {
        serde_json::to_string(payload)
            .map_err(|e| Error::Protocol(format!("Webhook JSON error: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_url_parsing() {
        let (url, sock) = parse_url("http://example.com/webhook");
        assert_eq!(url, "http://example.com/webhook");
        assert_eq!(sock, "");

        let (url_sock, sock_path) = parse_url("/var/run/webhook.sock:/api/v1");
        assert_eq!(url_sock, "http://localhost/api/v1");
        assert_eq!(sock_path, "/var/run/webhook.sock");

        let (url_abs, sock_abs) = parse_url("@my_abstract_socket:/hook");
        assert_eq!(url_abs, "http://localhost/hook");
        assert_eq!(sock_abs, "@my_abstract_socket");
    }

    #[test]
    fn test_webhook_deduplication() {
        let cfg = WebhookConfig {
            url: "http://127.0.0.1:8080/hook".to_string(),
            headers: HashMap::new(),
            deduplication: 60,
        };
        let notifier = WebhookNotifier::new(&cfg).unwrap().unwrap();

        assert!(!notifier.is_duplicate("user@example.com"));
        assert!(notifier.is_duplicate("user@example.com"));
        assert!(!notifier.is_duplicate("other@example.com"));
    }

    #[test]
    fn test_webhook_event_serialization() {
        let ev = WebhookNotifier::build_event(
            "proxy_out",
            Some("socks_in"),
            Some("alice@example.com"),
            Some("tcp"),
            Some("127.0.0.1:1234"),
            Some("example.com:443"),
        );
        let json = serde_json::to_string(&ev).unwrap();
        assert!(json.contains("proxy_out"));
        assert!(json.contains("alice@example.com"));
        assert!(json.contains("example.com:443"));
    }
}
