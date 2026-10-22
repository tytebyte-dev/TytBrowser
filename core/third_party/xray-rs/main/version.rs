// Module: main\\version.rs
// 1:1 Rust implementation corresponding to Go main\\version.go

pub const VERSION: &str = "26.3.27";

pub fn version() -> &'static str {
    VERSION
}

pub fn print_version() -> String {
    format!("Xray-core-rust {}", version())
}
