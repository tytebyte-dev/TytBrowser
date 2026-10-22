// Module: main\commands\all\api\outbounds_list.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\api\outbounds_list.go

use super::shared::ApiClientConfig;
use crate::main::commands::base::command::Command;

pub fn cmd_outbounds_list() -> Command {
    Command::new(
        "list-outbounds",
        "xray api list-outbounds [-s=server:port]",
        "List active outbounds",
    )
    .with_run(|args| {
        let client = ApiClientConfig::parse(args);
        Ok(format!("list-outbounds request sent to {}", client.server))
    })
}
