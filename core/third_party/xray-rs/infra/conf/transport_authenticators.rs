// Module: infra\conf\transport_authenticators.rs
// 1:1 Rust implementation corresponding to Go infra\conf\transport_authenticators.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthenticatorConfig {
    pub r#type: String,
    pub settings: Option<String>,
}

impl AuthenticatorConfig {
    pub fn new(t: &str) -> Self {
        Self {
            r#type: t.to_string(),
            settings: None,
        }
    }
}
