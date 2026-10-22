// Module: common\log\log.pb.rs
// 1:1 Rust implementation corresponding to Go common\log\log.pb.go

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(i32)]
#[derive(Default)]
pub enum Severity {
    Unknown = 0,
    Error = 1,
    Warning = 2,
    #[default]
    Info = 3,
    Debug = 4,
}

impl Severity {
    pub const SEVERITY_UNKNOWN: Severity = Severity::Unknown;
    pub const SEVERITY_ERROR: Severity = Severity::Error;
    pub const SEVERITY_WARNING: Severity = Severity::Warning;
    pub const SEVERITY_INFO: Severity = Severity::Info;
    pub const SEVERITY_DEBUG: Severity = Severity::Debug;

    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Unknown => "Unknown",
            Severity::Error => "Error",
            Severity::Warning => "Warning",
            Severity::Info => "Info",
            Severity::Debug => "Debug",
        }
    }

    pub fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Severity::Unknown),
            1 => Some(Severity::Error),
            2 => Some(Severity::Warning),
            3 => Some(Severity::Info),
            4 => Some(Severity::Debug),
            _ => None,
        }
    }

    pub fn number(&self) -> i32 {
        *self as i32
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
