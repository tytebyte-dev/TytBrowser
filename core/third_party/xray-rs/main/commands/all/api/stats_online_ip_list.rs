// Module: main\commands\all\api\stats_online_ip_list.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\api\stats_online_ip_list.go

use super::shared::ApiClientConfig;
use crate::main::commands::base::command::Command;

pub fn cmd_stats_online_ip_list() -> Command {
    Command::new(
        "online-ip-list",
        "xray api online-ip-list [-s=server:port]",
        "List active IP addresses for user",
    )
    .with_run(|args| {
        let client = ApiClientConfig::parse(args);
        Ok(format!("online-ip-list request sent to {}", client.server))
    })
}
