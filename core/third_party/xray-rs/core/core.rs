// Module: core\core.rs
// 1:1 Rust implementation corresponding to Go core\core.go

pub const VERSION_X: u8 = 26;
pub const VERSION_Y: u8 = 3;
pub const VERSION_Z: u8 = 27;

pub const BUILD: &str = "Custom";
pub const CODENAME: &str = "Xray, Penetrates Everything.";
pub const INTRO: &str = "A unified platform for anti-censorship.";

pub fn version() -> String {
    format!("{}.{}.{}", VERSION_X, VERSION_Y, VERSION_Z)
}

pub fn version_statement() -> Vec<String> {
    let v_str = format!(
        "Xray {} ({}) {} (rustc {}/{})",
        version(),
        CODENAME,
        BUILD,
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    vec![v_str, INTRO.to_string()]
}

pub use super::xray::{Instance, Server};
