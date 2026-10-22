// Module: main\commands\base\command.rs
// 1:1 Rust implementation corresponding to Go main\commands\base\command.go

use crate::common::errors::Result;
use std::sync::Arc;

pub type CommandFn = Arc<dyn Fn(&[&str]) -> Result<String> + Send + Sync>;

#[derive(Clone)]
pub struct Command {
    pub name: String,
    pub usage: String,
    pub short: String,
    pub long: String,
    pub subcommands: Vec<Command>,
    pub run: Option<CommandFn>,
}

impl Command {
    pub fn new(
        name: impl Into<String>,
        usage: impl Into<String>,
        short: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            usage: usage.into(),
            short: short.into(),
            long: String::new(),
            subcommands: Vec::new(),
            run: None,
        }
    }

    pub fn with_long(mut self, long: impl Into<String>) -> Self {
        self.long = long.into();
        self
    }

    pub fn with_subcommands(mut self, subcommands: Vec<Command>) -> Self {
        self.subcommands = subcommands;
        self
    }

    pub fn with_run<F>(mut self, f: F) -> Self
    where
        F: Fn(&[&str]) -> Result<String> + Send + Sync + 'static,
    {
        self.run = Some(Arc::new(f));
        self
    }

    pub fn execute(&self, args: &[&str]) -> Result<String> {
        if !args.is_empty() {
            let sub = args[0];
            for cmd in &self.subcommands {
                if cmd.name == sub {
                    return cmd.execute(&args[1..]);
                }
            }
        }
        if let Some(run_fn) = &self.run {
            run_fn(args)
        } else {
            Ok(format!("Usage: {}\n\n{}", self.usage, self.short))
        }
    }
}
