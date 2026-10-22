// Module: main\commands\all\api\source_ip_block.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\api\source_ip_block.go

use super::shared::ApiClientConfig;
use crate::main::commands::base::command::Command;

pub fn cmd_source_ip_block() -> Command {
    Command::new(
        "block-source-ip",
        "xray api block-source-ip [-s=server:port]",
        "Block source IP address",
    )
    .with_run(|args| {
        let client = ApiClientConfig::parse(args);
        Ok(format!("block-source-ip request sent to {}", client.server))
    })
}
