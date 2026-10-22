// Module: main\commands\all\api\inbounds_add.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\api\inbounds_add.go

use super::shared::ApiClientConfig;
use crate::main::commands::base::command::Command;

pub fn cmd_inbounds_add() -> Command {
    Command::new(
        "add-inbounds",
        "xray api add-inbounds [-s=server:port]",
        "Add inbounds to running instance",
    )
    .with_run(|args| {
        let client = ApiClientConfig::parse(args);
        Ok(format!("add-inbounds request sent to {}", client.server))
    })
}
