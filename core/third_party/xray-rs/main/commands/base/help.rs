// Module: main\\commands\\base\\help.rs
// 1:1 Rust implementation corresponding to Go main\\commands\\base\\help.go

use super::command::Command;

pub fn generate_help_text(cmd: &Command) -> String {
    format!("{}\nUsage: {}\n", cmd.short, cmd.usage)
}
