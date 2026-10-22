// Module: main\commands\base\env.rs
// 1:1 Rust implementation corresponding to Go main\commands\base\env.go

use std::env;

pub fn get_env_var(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}
