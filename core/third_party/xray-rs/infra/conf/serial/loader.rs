// Module: infra\conf\serial\loader.rs
// 1:1 Rust implementation corresponding to Go infra\conf\serial\loader.go

use crate::common::errors::{Error, Result};
use std::fs;
use std::path::Path;

pub fn load_config_file<P: AsRef<Path>>(path: P) -> Result<String> {
    fs::read_to_string(path).map_err(Error::Io)
}
