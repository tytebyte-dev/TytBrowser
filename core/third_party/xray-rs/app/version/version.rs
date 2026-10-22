// Module: app\version\version.rs
// 1:1 Rust implementation corresponding to Go app\version\version.go

use super::compare_versions;
use super::config_pb::Config;
use crate::common::errors::{Error, Result};

pub const VERSION: &str = "26.3.27";

#[derive(Debug, Clone)]
pub struct Version {
    pub config: Config,
}

impl Version {
    pub fn new(config: Config) -> Result<Self> {
        let core_version = if config.core_version.is_empty() {
            VERSION
        } else {
            &config.core_version
        };
        if !config.min_version.is_empty() {
            let result = compare_versions(&config.min_version, core_version)?;
            if result > 0 {
                return Err(Error::Config(format!(
                    "this config must be run on version {} or higher",
                    config.min_version
                )));
            }
        }
        if !config.max_version.is_empty() {
            let result = compare_versions(&config.max_version, core_version)?;
            if result < 0 {
                return Err(Error::Config(format!(
                    "this config should be run on version {} or lower",
                    config.max_version
                )));
            }
        }
        Ok(Self { config })
    }
}
