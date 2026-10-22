use crate::common::errors::{Error, Result};
use crate::infra::conf::Config;
use serde_json::Value;
use std::path::Path;

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load_from_json_str(json_str: &str) -> Result<Config> {
        serde_json::from_str::<Config>(json_str)
            .map_err(|e| Error::Config(format!("JSON parsing error: {}", e)))
    }

    pub fn load_from_json_value(val: Value) -> Result<Config> {
        serde_json::from_value::<Config>(val)
            .map_err(|e| Error::Config(format!("JSON value conversion error: {}", e)))
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;
        Self::load_from_json_str(&content)
    }
}
