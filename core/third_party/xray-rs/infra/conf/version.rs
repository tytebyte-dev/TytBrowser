// Module: infra\conf\version.rs
// 1:1 Rust implementation corresponding to Go infra\conf\version.go

use super::buildable::Buildable;
use crate::app::version::VERSION as CORE_VERSION;
use crate::app::version::VersionConfig as ProtoVersionConfig;
use crate::common::errors::Result;
use serde::{Deserialize, Serialize};

pub const CONFIG_VERSION: &str = "1.0";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionConfig {
    #[serde(default)]
    pub min: String,
    #[serde(default)]
    pub max: String,
}

impl Buildable for VersionConfig {
    type Output = ProtoVersionConfig;

    fn build(self) -> Result<Self::Output> {
        Ok(ProtoVersionConfig {
            core_version: CORE_VERSION.to_string(),
            min_version: self.min,
            max_version: self.max,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_config_build() {
        let v_conf = VersionConfig {
            min: "25.0.0".into(),
            max: "27.0.0".into(),
        };
        let proto = v_conf.build().unwrap();
        assert_eq!(proto.core_version, CORE_VERSION);
        assert_eq!(proto.min_version, "25.0.0");
        assert_eq!(proto.max_version, "27.0.0");
    }
}
