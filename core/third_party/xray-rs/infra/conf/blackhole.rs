// Module: infra\conf\blackhole.rs
// 1:1 Rust implementation corresponding to Go infra\conf\blackhole.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoneResponse;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpResponse;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlackholeConfig {
    #[serde(default)]
    pub response: Option<serde_json::Value>,
}

impl BlackholeConfig {
    pub fn with_none_response() -> Self {
        Self {
            response: Some(serde_json::json!({"type": "none"})),
        }
    }

    pub fn with_http_response() -> Self {
        Self {
            response: Some(serde_json::json!({"type": "http"})),
        }
    }

    pub fn response_type(&self) -> Option<&str> {
        self.response
            .as_ref()
            .and_then(|v| v.get("type"))
            .and_then(|t| t.as_str())
    }
}
