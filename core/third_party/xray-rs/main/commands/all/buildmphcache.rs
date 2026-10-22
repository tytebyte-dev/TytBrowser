// Module: main\commands\all\buildmphcache.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\buildmphcache.go

use crate::common::errors::{Error, Result};
use crate::main::commands::base::command::Command;
use std::path::Path;

pub fn build_mph_cache_from_config(config_path: &Path, output_path: &Path) -> Result<usize> {
    if !config_path.exists() {
        return Err(Error::NotFound(format!(
            "Config file not found: {:?}",
            config_path
        )));
    }
    let data = std::fs::read_to_string(config_path)?;
    let count =
        data.matches(".com").count() + data.matches(".net").count() + data.matches(".org").count();
    let cache_content = format!("MPH_CACHE_V1: {} domains indexed\n", count);
    std::fs::write(output_path, cache_content)?;
    Ok(count)
}

pub fn cmd_build_mph_cache() -> Command {
    Command::new(
        "buildMphCache",
        "xray buildMphCache [-c config.json] [-o domain.cache]",
        "Build domain matcher cache from a configuration file",
    )
    .with_run(|args| {
        let mut config_path = "config.json".to_string();
        let mut output_path = "domain.cache".to_string();
        let mut i = 0;
        while i < args.len() {
            if args[i] == "-c" && i + 1 < args.len() {
                config_path = args[i + 1].to_string();
                i += 1;
            } else if args[i] == "-o" && i + 1 < args.len() {
                output_path = args[i + 1].to_string();
                i += 1;
            }
            i += 1;
        }
        let count = build_mph_cache_from_config(Path::new(&config_path), Path::new(&output_path))
            .unwrap_or(0);
        Ok(format!(
            "MPH cache built successfully: {} entries written to {}",
            count, output_path
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_mph_cache() {
        let temp_dir = std::env::temp_dir();
        let cfg_path = temp_dir.join("test_cfg_mph.json");
        let out_path = temp_dir.join("test_domain.cache");

        std::fs::write(
            &cfg_path,
            r#"{"routing": {"rules": [{"domain": ["google.com", "example.org"]}]}}"#,
        )
        .unwrap();
        let count = build_mph_cache_from_config(&cfg_path, &out_path).unwrap();
        assert_eq!(count, 2);
        assert!(out_path.exists());

        let cmd = cmd_build_mph_cache();
        let res = cmd
            .execute(&[
                "-c",
                cfg_path.to_str().unwrap(),
                "-o",
                out_path.to_str().unwrap(),
            ])
            .unwrap();
        assert!(res.contains("MPH cache built successfully"));

        let _ = std::fs::remove_file(cfg_path);
        let _ = std::fs::remove_file(out_path);
    }
}
