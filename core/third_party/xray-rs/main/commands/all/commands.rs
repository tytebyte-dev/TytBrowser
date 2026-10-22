// Module: main\commands\all\commands.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\commands.go

use super::api::api::cmd_api;
use super::buildmphcache::cmd_build_mph_cache;
use super::convert::convert::cmd_convert;
use super::mldsa65::cmd_mldsa65;
use super::mlkem768::cmd_mlkem768;
use super::tls::cmd_tls;
use super::uuid::cmd_uuid;
use super::vlessenc::cmd_vlessenc;
use super::wg::cmd_wg;
use super::x25519::cmd_x25519;
use crate::main::commands::base::command::Command;

pub fn all_commands() -> Vec<Command> {
    vec![
        cmd_uuid(),
        cmd_x25519(),
        cmd_wg(),
        cmd_mldsa65(),
        cmd_mlkem768(),
        cmd_vlessenc(),
        cmd_build_mph_cache(),
        cmd_api(),
        cmd_tls(),
        cmd_convert(),
    ]
}
