// Module: main\commands\all\uuid.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\uuid.go

use crate::main::commands::base::command::Command;
use uuid::Uuid;

pub fn cmd_uuid() -> Command {
    Command::new("uuid", "xray uuid", "Generate a new random UUID (v4)").with_run(|_args| {
        let id = Uuid::new_v4();
        Ok(id.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_command() {
        let cmd = cmd_uuid();
        let out = cmd.execute(&[]).unwrap();
        assert_eq!(out.len(), 36);
        assert!(Uuid::parse_str(&out).is_ok());
    }
}
