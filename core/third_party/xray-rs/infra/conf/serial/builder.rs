// Module: infra\conf\serial\builder.rs
// 1:1 Rust implementation corresponding to Go infra\conf\serial\builder.go

use crate::common::errors::{Error, Result};
use crate::infra::conf::Config;
use serde_json::Value;

pub fn build_json_config(raw_json: &str) -> Result<Value> {
    serde_json::from_str(raw_json).map_err(|e| Error::Config(e.to_string()))
}

pub fn merge_configs(configs: &[Config]) -> Config {
    let mut merged = Config::default();
    for c in configs {
        if let Some(ref l) = c.log {
            merged.log = Some(l.clone());
        }
        if let Some(ref d) = c.dns {
            merged.dns = Some(d.clone());
        }
        if let Some(ref r) = c.routing {
            merged.routing = Some(r.clone());
        }
        merged.inbounds.extend(c.inbounds.clone());
        merged.outbounds.extend(c.outbounds.clone());
    }
    merged
}

pub fn build_config_from_slices(raw_jsons: &[&str]) -> Result<Config> {
    let mut configs = Vec::new();
    for raw in raw_jsons {
        let conf: Config = serde_json::from_str(raw)
            .map_err(|e| Error::Config(format!("Failed to parse config: {}", e)))?;
        configs.push(conf);
    }
    Ok(merge_configs(&configs))
}
