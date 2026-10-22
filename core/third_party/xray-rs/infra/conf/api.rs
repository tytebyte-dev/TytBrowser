// Module: infra\conf\api.rs
// 1:1 Rust implementation corresponding to Go infra\conf\api.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiConfig {
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub services: Vec<String>,
}

impl ApiConfig {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            services: Vec::new(),
        }
    }

    pub fn with_service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }
}
