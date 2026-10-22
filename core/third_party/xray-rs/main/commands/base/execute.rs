// Module: main\\commands\\base\\execute.rs
// 1:1 Rust implementation corresponding to Go main\\commands\\base\\execute.go

use super::command::Command;

pub fn execute_command(cmd: &Command, args: &[&str]) -> Result<String, String> {
    cmd.execute(args).map_err(|e| e.to_string())
}
