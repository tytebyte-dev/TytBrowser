// Module: main\commands\all\convert\protobuf.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\convert\protobuf.go

use crate::common::errors::{Error, Result};
use crate::infra::conf::serial::builder::build_config_from_slices;
use crate::main::commands::base::command::Command;

pub const PROTOBUF_FORMAT: &str = "pb";

pub fn convert_configs_to_pb_json(raw_jsons: &[&str]) -> Result<String> {
    let merged = build_config_from_slices(raw_jsons)?;
    serde_json::to_string_pretty(&merged)
        .map_err(|e| Error::Config(format!("Serialization error: {}", e)))
}

pub fn cmd_convert_pb() -> Command {
    Command::new(
        "pb",
        "xray convert pb [-o output.pb] [-debug] [config files...]",
        "Convert multiple json configs to protobuf/merged config",
    )
    .with_run(|args| {
        let mut opt_file = None;
        let mut opt_debug = false;
        let mut configs = Vec::new();

        let mut i = 0;
        while i < args.len() {
            if (args[i] == "-o" || args[i] == "-outpbfile") && i + 1 < args.len() {
                opt_file = Some(args[i + 1]);
                i += 1;
            } else if args[i] == "-d" || args[i] == "-debug" {
                opt_debug = true;
            } else if !args[i].starts_with('-') {
                if let Ok(c) = std::fs::read_to_string(args[i]) {
                    configs.push(c);
                } else {
                    configs.push(args[i].to_string());
                }
            }
            i += 1;
        }

        if configs.is_empty() {
            return Err(Error::Config("invalid config list length: 0".into()));
        }

        let slice: Vec<&str> = configs.iter().map(|s| s.as_str()).collect();
        let formatted = convert_configs_to_pb_json(&slice)?;

        if let Some(out_path) = opt_file {
            std::fs::write(out_path, formatted.as_bytes()).map_err(Error::Io)?;
            Ok(format!("Output ProtoBuf file is {}", out_path))
        } else if opt_debug {
            Ok(formatted)
        } else {
            Ok(format!("Merged {} configs successfully", configs.len()))
        }
    })
}
