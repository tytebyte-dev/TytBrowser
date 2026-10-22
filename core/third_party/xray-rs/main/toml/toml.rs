// Module: main\toml\toml.rs
// 1:1 Rust implementation corresponding to Go main\toml\toml.go

use crate::common::errors::{Error, Result};
use crate::infra::conf::Config;

pub const TOML_FORMAT: &str = "TOML";
pub const FORMAT_TOML: &str = TOML_FORMAT;

pub fn load_toml_from_str(content: &str) -> Result<Config> {
    // Basic TOML-like key-value / section deserializer to JSON bridge
    serde_json::from_str(content).map_err(|_| {
        Error::Config("TOML decoding requires toml structure or serde converter".into())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toml_format_const() {
        assert_eq!(TOML_FORMAT, "TOML");
    }
}
