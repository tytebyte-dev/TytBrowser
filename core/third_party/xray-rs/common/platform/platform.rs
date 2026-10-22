// Module: common\platform\platform.rs
// 1:1 Rust implementation corresponding to Go common\platform\platform.go

use std::env;
use std::path::PathBuf;

pub const CONFIG_LOCATION: &str = "xray.location.config";
pub const CONFDIR_LOCATION: &str = "xray.location.confdir";
pub const ASSET_LOCATION: &str = "xray.location.asset";
pub const CERT_LOCATION: &str = "xray.location.cert";

pub const USE_READ_V: &str = "xray.buf.readv";
pub const USE_FREEDOM_SPLICE: &str = "xray.buf.splice";
pub const USE_VMESS_PADDING: &str = "xray.vmess.padding";
pub const USE_CONE: &str = "xray.cone.disabled";

pub const BUFFER_SIZE: &str = "xray.ray.buffer.size";
pub const BROWSER_DIALER_ADDRESS: &str = "xray.browser.dialer";
pub const XUDP_LOG: &str = "xray.xudp.show";
pub const XUDP_BASE_KEY: &str = "xray.xudp.basekey";

pub const TUN_FD_KEY: &str = "xray.tun.fd";
pub const MPH_CACHE_PATH: &str = "xray.mph.cache";

pub fn normalize_env_name(name: &str) -> String {
    name.trim().replace('.', "_").to_uppercase()
}

#[derive(Debug, Clone)]
pub struct EnvFlag {
    pub name: String,
    pub alt_name: String,
}

impl EnvFlag {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            alt_name: normalize_env_name(name),
        }
    }

    pub fn get_value<F: FnOnce() -> String>(&self, default_value: F) -> String {
        if let Ok(v) = env::var(&self.name) {
            return v;
        }
        if !self.alt_name.is_empty()
            && let Ok(v) = env::var(&self.alt_name)
        {
            return v;
        }
        default_value()
    }

    pub fn get_value_str(&self, default_val: &str) -> String {
        self.get_value(|| default_val.to_string())
    }

    pub fn get_value_as_int(&self, default_val: i32) -> i32 {
        let s = self.get_value(|| "".to_string());
        if s.is_empty() {
            return default_val;
        }
        s.parse().unwrap_or(default_val)
    }

    pub fn get_value_as_bool(&self, default_val: bool) -> bool {
        let s = self.get_value(|| "".to_string());
        if s.is_empty() {
            return default_val;
        }
        match s.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => true,
            "false" | "0" | "no" | "off" => false,
            _ => default_val,
        }
    }
}

pub fn get_executable_dir() -> PathBuf {
    if let Ok(exe) = env::current_exe()
        && let Some(parent) = exe.parent()
    {
        return parent.to_path_buf();
    }
    PathBuf::from(".")
}

pub fn get_configuration_path() -> PathBuf {
    let flag = EnvFlag::new(CONFIG_LOCATION);
    let config_dir = flag.get_value(|| get_executable_dir().to_string_lossy().into_owned());
    PathBuf::from(config_dir).join("config.json")
}

pub fn get_conf_dir_path() -> String {
    let flag = EnvFlag::new(CONFDIR_LOCATION);
    flag.get_value(String::new)
}

pub fn get_asset_location(file: &str) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        crate::common::platform::windows::get_asset_location(file)
    }
    #[cfg(not(target_os = "windows"))]
    {
        crate::common::platform::others::get_asset_location(file)
    }
}

pub fn get_cert_location(file: &str) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        crate::common::platform::windows::get_cert_location(file)
    }
    #[cfg(not(target_os = "windows"))]
    {
        crate::common::platform::others::get_cert_location(file)
    }
}

pub fn line_separator() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        crate::common::platform::windows::line_separator()
    }
    #[cfg(not(target_os = "windows"))]
    {
        crate::common::platform::others::line_separator()
    }
}

pub fn get_os_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(target_os = "linux")]
    {
        "linux"
    }
    #[cfg(target_os = "macos")]
    {
        "darwin"
    }
    #[cfg(target_os = "freebsd")]
    {
        "freebsd"
    }
    #[cfg(not(any(
        target_os = "windows",
        target_os = "linux",
        target_os = "macos",
        target_os = "freebsd"
    )))]
    {
        "unknown"
    }
}
