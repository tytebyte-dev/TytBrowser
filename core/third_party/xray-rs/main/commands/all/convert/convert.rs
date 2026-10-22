// Module: main\commands\all\convert\convert.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\convert\convert.go

use super::json::cmd_convert_json;
use super::protobuf::cmd_convert_pb;
use crate::main::commands::base::command::Command;

pub const CONVERT_COMMAND_NAME: &str = "convert";

pub fn cmd_convert() -> Command {
    Command::new("convert", "xray convert [subcommand]", "Convert configs")
        .with_subcommands(vec![cmd_convert_json(), cmd_convert_pb()])
}
