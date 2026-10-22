// Module: main\commands\all\api\api.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\api\api.go

use super::balancer_info::cmd_balancer_info;
use super::balancer_override::cmd_balancer_override;
use super::inbound_user::cmd_inbound_user;
use super::inbound_user_add::cmd_inbound_user_add;
use super::inbound_user_count::cmd_inbound_user_count;
use super::inbound_user_remove::cmd_inbound_user_remove;
use super::inbounds_add::cmd_inbounds_add;
use super::inbounds_list::cmd_inbounds_list;
use super::inbounds_remove::cmd_inbounds_remove;
use super::logger_restart::cmd_logger_restart;
use super::outbounds_add::cmd_outbounds_add;
use super::outbounds_list::cmd_outbounds_list;
use super::outbounds_remove::cmd_outbounds_remove;
use super::rules_add::cmd_rules_add;
use super::rules_list::cmd_rules_list;
use super::rules_remove::cmd_rules_remove;
use super::source_ip_block::cmd_source_ip_block;
use super::stats_get::cmd_stats_get;
use super::stats_get_all_online_users::cmd_stats_get_all_online_users;
use super::stats_online::cmd_stats_online;
use super::stats_online_ip_list::cmd_stats_online_ip_list;
use super::stats_query::cmd_stats_query;
use super::stats_sys::cmd_stats_sys;
use crate::main::commands::base::command::Command;

pub fn cmd_api() -> Command {
    Command::new(
        "api",
        "xray api <subcommand>",
        "Call an API in an Xray process",
    )
    .with_subcommands(vec![
        cmd_balancer_info(),
        cmd_balancer_override(),
        cmd_inbound_user(),
        cmd_inbound_user_add(),
        cmd_inbound_user_count(),
        cmd_inbound_user_remove(),
        cmd_inbounds_add(),
        cmd_inbounds_list(),
        cmd_inbounds_remove(),
        cmd_logger_restart(),
        cmd_outbounds_add(),
        cmd_outbounds_list(),
        cmd_outbounds_remove(),
        cmd_rules_add(),
        cmd_rules_list(),
        cmd_rules_remove(),
        cmd_source_ip_block(),
        cmd_stats_get(),
        cmd_stats_get_all_online_users(),
        cmd_stats_online(),
        cmd_stats_online_ip_list(),
        cmd_stats_query(),
        cmd_stats_sys(),
    ])
}
