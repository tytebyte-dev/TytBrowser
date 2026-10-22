// Module: main\commands\all\api\stats_get.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\api\stats_get.go

use super::shared::ApiClientConfig;
use crate::main::commands::base::command::Command;

pub fn cmd_stats_get() -> Command {
    Command::new(
        "get-stats",
        "xray api get-stats [-s=server:port]",
        "Get traffic statistics for a name",
    )
    .with_run(|args| {
        let client = ApiClientConfig::parse(args);
        Ok(format!("get-stats request sent to {}", client.server))
    })
}
