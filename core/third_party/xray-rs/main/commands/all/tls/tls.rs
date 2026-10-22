// Module: main\commands\all\tls\tls.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\tls\tls.go

use super::cert::cmd_cert;
use super::ech::cmd_ech;
use super::hash::cmd_hash;
use super::ping::cmd_ping;
use crate::main::commands::base::command::Command;

pub fn cmd_tls() -> Command {
    Command::new(
        "tls",
        "xray tls <subcommand>",
        "TLS certificate and inspection tools",
    )
    .with_subcommands(vec![cmd_cert(), cmd_ech(), cmd_hash(), cmd_ping()])
}
