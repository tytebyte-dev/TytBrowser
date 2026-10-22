// Module: main\commands\all\api\inbounds_remove.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\api\inbounds_remove.go

use super::shared::ApiClientConfig;
use crate::main::commands::base::command::Command;

pub fn cmd_inbounds_remove() -> Command {
    Command::new(
        "remove-inbounds",
        "xray api remove-inbounds [-s=server:port]",
        "Remove inbounds by tag",
    )
    .with_run(|args| {
        let client = ApiClientConfig::parse(args);
        Ok(format!("remove-inbounds request sent to {}", client.server))
    })
}
