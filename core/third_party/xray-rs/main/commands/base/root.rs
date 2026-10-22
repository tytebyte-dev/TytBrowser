// Module: main\commands\base\root.rs
// 1:1 Rust implementation corresponding to Go main\commands\base\root.go

use super::command::Command;

pub fn create_root_command() -> Command {
    Command::new(
        "xray",
        "xray <command> [arguments]",
        "Xray core proxy and routing tool",
    )
}
