pub mod command;
#[path = "config.pb.rs"]
pub mod config_pb;
pub mod log;
pub mod log_creator;

#[cfg(test)]
pub mod log_test;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::atomic::{AtomicU8, Ordering};
use tracing_subscriber::{EnvFilter, fmt};

pub use log_creator::{LogCreatorOptions, LogType, create_logger};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    None = 0,
    Error = 1,
    Warning = 2,
    Info = 3,
    Debug = 4,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warning" | "warn" => LogLevel::Warning,
            "error" => LogLevel::Error,
            "none" => LogLevel::None,
            _ => LogLevel::Warning,
        }
    }
}

use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MultiWriter {
    file: Option<Arc<Mutex<std::fs::File>>>,
}

impl MultiWriter {
    pub fn new(path: Option<&str>) -> Self {
        let file = path.and_then(|p| {
            if let Some(parent) = std::path::Path::new(p).parent()
                && !parent.as_os_str().is_empty()
            {
                let _ = std::fs::create_dir_all(parent);
            }
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(p)
                .ok()
                .map(|f| Arc::new(Mutex::new(f)))
        });
        Self { file }
    }
}

impl Write for MultiWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let _ = std::io::stdout().write_all(buf);
        if let Some(ref f) = self.file
            && let Ok(mut file) = f.lock()
        {
            let _ = file.write_all(buf);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let _ = std::io::stdout().flush();
        if let Some(ref f) = self.file
            && let Ok(mut file) = f.lock()
        {
            let _ = file.flush();
        }
        Ok(())
    }
}

pub fn init_logger(level_str: &str) {
    init_logger_ext(level_str, None);
}

pub fn init_logger_ext(level_str: &str, file_path: Option<&str>) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level_str));

    if let Some(path) = file_path {
        let writer = MultiWriter::new(Some(path));
        let _ = fmt()
            .with_env_filter(filter)
            .with_target(false)
            .with_ansi(false)
            .with_writer(move || writer.clone())
            .try_init();
    } else {
        let _ = fmt().with_env_filter(filter).with_target(false).try_init();
    }
}

pub struct LogManager {
    level: AtomicU8,
    mask4: u8,
    mask6: u8,
}

impl LogManager {
    pub fn new(level: LogLevel, mask4: u8, mask6: u8) -> Self {
        Self {
            level: AtomicU8::new(level as u8),
            mask4,
            mask6,
        }
    }

    pub fn start(&self) {
        // Start or restart logger
    }

    pub fn get_level(&self) -> LogLevel {
        match self.level.load(Ordering::Relaxed) {
            0 => LogLevel::None,
            1 => LogLevel::Error,
            2 => LogLevel::Warning,
            3 => LogLevel::Info,
            4 => LogLevel::Debug,
            _ => LogLevel::Warning,
        }
    }

    pub fn set_level(&self, level: LogLevel) {
        self.level.store(level as u8, Ordering::Relaxed);
    }

    pub fn mask_ip(&self, ip: &IpAddr) -> IpAddr {
        match ip {
            IpAddr::V4(v4) => {
                if self.mask4 == 0 {
                    return IpAddr::V4(*v4);
                }
                let octets = v4.octets();
                let shift = self.mask4.min(32);
                let mask = !((1u32 << (32 - shift)) - 1);
                let masked = u32::from_be_bytes(octets) & mask;
                IpAddr::V4(Ipv4Addr::from(masked))
            }
            IpAddr::V6(v6) => {
                if self.mask6 == 0 {
                    return IpAddr::V6(*v6);
                }
                let octets = v6.octets();
                let shift = self.mask6.min(128);
                let mask = !((1u128 << (128 - shift)) - 1);
                let masked = u128::from_be_bytes(octets) & mask;
                IpAddr::V6(Ipv6Addr::from(masked))
            }
        }
    }

    pub fn format_access_log(
        &self,
        from_ip: Option<&IpAddr>,
        dest: &str,
        status: &str,
        outbound: &str,
    ) -> String {
        let from_str = match from_ip {
            Some(ip) => self.mask_ip(ip).to_string(),
            None => "unknown".to_string(),
        };
        format!("{} [{}] {} -> [{}]", from_str, status, dest, outbound)
    }
}
