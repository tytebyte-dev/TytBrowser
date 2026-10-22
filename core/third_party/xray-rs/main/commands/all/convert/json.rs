// Module: main\commands\all\convert\json.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\convert\json.go

use crate::common::errors::{Error, Result};
use crate::main::commands::base::command::Command;

pub fn format_json(raw: &str) -> Result<String> {
    let v: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| Error::Config(format!("Invalid JSON: {}", e)))?;
    serde_json::to_string_pretty(&v).map_err(|e| Error::Config(format!("Format JSON error: {}", e)))
}

pub fn cmd_convert_json() -> Command {
    Command::new(
        "json",
        "xray convert json [-t] [json content or file]",
        "Convert typedMessage or raw JSON to formatted json",
    )
    .with_run(|args| {
        if args.is_empty() {
            return Err(Error::Config("empty input list".into()));
        }
        let input = args.last().unwrap();
        let content = if std::path::Path::new(input).exists() {
            std::fs::read_to_string(input).map_err(Error::Io)?
        } else {
            input.to_string()
        };
        format_json(&content)
    })
}
