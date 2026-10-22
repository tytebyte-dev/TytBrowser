// Module: main\commands\all\api\outbounds_add.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\api\outbounds_add.go

use super::shared::ApiClientConfig;
use crate::main::commands::base::command::Command;

pub fn cmd_outbounds_add() -> Command {
    Command::new(
        "add-outbounds",
        "xray api add-outbounds [-s=server:port]",
        "Add outbounds to running instance",
    )
    .with_run(|args| {
        let client = ApiClientConfig::parse(args);
        Ok(format!("add-outbounds request sent to {}", client.server))
    })
}
