// Module: main\commands\all\api\inbound_user.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\api\inbound_user.go

use super::shared::ApiClientConfig;
use crate::main::commands::base::command::Command;

pub fn cmd_inbound_user() -> Command {
    Command::new(
        "inbound-user",
        "xray api inbound-user [-s=server:port]",
        "Inspect users in an inbound",
    )
    .with_run(|args| {
        let client = ApiClientConfig::parse(args);
        Ok(format!("inbound-user request sent to {}", client.server))
    })
}
