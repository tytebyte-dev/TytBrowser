// Module: infra\conf\metrics.rs
// 1:1 Rust implementation corresponding to Go infra\conf\metrics.go

use crate::app::metrics::Config as MetricsConfigProto;
use crate::common::errors::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricsConfig {
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub listen: String,
}

impl MetricsConfig {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            listen: String::new(),
        }
    }

    pub fn build(&self) -> Result<MetricsConfigProto> {
        if self.listen.is_empty() && self.tag.is_empty() {
            return Err(Error::Config(
                "Metrics must have a tag or listen address.".into(),
            ));
        }
        let tag = if self.tag.is_empty() {
            "Metrics".to_string()
        } else {
            self.tag.clone()
        };
        Ok(MetricsConfigProto {
            tag,
            listen: self.listen.clone(),
        })
    }
}
