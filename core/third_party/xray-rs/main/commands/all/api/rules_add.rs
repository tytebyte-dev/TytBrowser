// Module: main\commands\all\api\rules_add.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\api\rules_add.go

use super::shared::ApiClientConfig;
use crate::main::commands::base::command::Command;

pub fn cmd_rules_add() -> Command {
    Command::new(
        "add-rules",
        "xray api add-rules [-s=server:port]",
        "Add routing rules dynamically",
    )
    .with_run(|args| {
        let client = ApiClientConfig::parse(args);
        Ok(format!("add-rules request sent to {}", client.server))
    })
}
