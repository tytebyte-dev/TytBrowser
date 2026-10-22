pub mod command;
pub mod env;
pub mod execute;
pub mod help;
pub mod root;

pub use command::{Command, CommandFn};
pub use env::get_env_var;
pub use execute::execute_command;
pub use help::generate_help_text;
pub use root::create_root_command;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_command_execution_and_help() {
        let root = create_root_command();
        assert_eq!(root.name, "xray");

        let help = generate_help_text(&root);
        assert!(help.contains("xray <command>"));

        let env_val = get_env_var("NON_EXISTENT_VAR_12345", "fallback");
        assert_eq!(env_val, "fallback");

        let sub = Command::new("test", "xray test", "Test command")
            .with_run(|_args| Ok("executed test".into()));

        let cmd = root.with_subcommands(vec![sub]);
        let res = execute_command(&cmd, &["test"]).unwrap();
        assert_eq!(res, "executed test");
    }
}
