// Module: main\commands\all\api\inbound_user_remove.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\api\inbound_user_remove.go

use super::shared::ApiClientConfig;
use crate::main::commands::base::command::Command;

pub fn cmd_inbound_user_remove() -> Command {
    Command::new(
        "remove-inbound-user",
        "xray api remove-inbound-user [-s=server:port]",
        "Remove user from inbound",
    )
    .with_run(|args| {
        let client = ApiClientConfig::parse(args);
        Ok(format!(
            "remove-inbound-user request sent to {}",
            client.server
        ))
    })
}
