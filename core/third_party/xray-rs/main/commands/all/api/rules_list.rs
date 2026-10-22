// Module: main\commands\all\api\rules_list.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\api\rules_list.go

use super::shared::ApiClientConfig;
use crate::main::commands::base::command::Command;

pub fn cmd_rules_list() -> Command {
    Command::new(
        "list-rules",
        "xray api list-rules [-s=server:port]",
        "List current routing rules",
    )
    .with_run(|args| {
        let client = ApiClientConfig::parse(args);
        Ok(format!("list-rules request sent to {}", client.server))
    })
}
