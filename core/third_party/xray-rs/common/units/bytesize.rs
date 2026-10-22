// Module: common\units\bytesize.rs
// 1:1 Rust implementation corresponding to Go common\units\bytesize.go

use crate::common::errors::{Error, Result};
use std::fmt;
use std::ops::Deref;

pub const KB: u64 = 1024;
pub const MB: u64 = 1024 * KB;
pub const GB: u64 = 1024 * MB;
pub const TB: u64 = 1024 * GB;
pub const PB: u64 = 1024 * TB;
pub const EB: u64 = 1024 * PB;

/// ByteSize represents a size in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ByteSize(pub u64);

impl ByteSize {
    pub const KB: ByteSize = ByteSize(KB);
    pub const MB: ByteSize = ByteSize(MB);
    pub const GB: ByteSize = ByteSize(GB);
    pub const TB: ByteSize = ByteSize(TB);
    pub const PB: ByteSize = ByteSize(PB);
    pub const EB: ByteSize = ByteSize(EB);

    pub fn new(val: u64) -> Self {
        ByteSize(val)
    }

    /// Parse parses ByteSize from string (e.g. "1.00KB", "2MB", "10GiB")
    pub fn parse(&mut self, s: &str) -> Result<()> {
        let s = s.trim().to_uppercase();
        let i = s
            .find(|c: char| c.is_alphabetic())
            .ok_or_else(|| Error::Protocol("invalid or unsupported unit".into()))?;
        let (bytes_str, multiple) = s.split_at(i);
        let bytes: f64 = bytes_str
            .trim()
            .parse()
            .map_err(|_| Error::Protocol("invalid size".into()))?;
        if bytes <= 0.0 {
            return Err(Error::Protocol("invalid size".into()));
        }

        let val = match multiple {
            "B" => bytes,
            "K" | "KB" | "KIB" => bytes * (KB as f64),
            "M" | "MB" | "MIB" => bytes * (MB as f64),
            "G" | "GB" | "GIB" => bytes * (GB as f64),
            "T" | "TB" | "TIB" => bytes * (TB as f64),
            "P" | "PB" | "PIB" => bytes * (PB as f64),
            "E" | "EB" | "EIB" => bytes * (EB as f64),
            _ => return Err(Error::Protocol("invalid or unsupported unit".into())),
        };

        self.0 = val as u64;
        Ok(())
    }

    pub fn to_string_formatted(&self) -> String {
        if self.0 == 0 {
            return "0".to_string();
        }
        let (val, unit) = if self.0 < KB {
            (self.0 as f64, "B")
        } else if self.0 < MB {
            (self.0 as f64 / KB as f64, "KB")
        } else if self.0 < GB {
            (self.0 as f64 / MB as f64, "MB")
        } else if self.0 < TB {
            (self.0 as f64 / GB as f64, "GB")
        } else if self.0 < PB {
            (self.0 as f64 / TB as f64, "TB")
        } else if self.0 < EB {
            (self.0 as f64 / PB as f64, "PB")
        } else {
            (self.0 as f64 / EB as f64, "EB")
        };

        let formatted = format!("{:.2}", val);
        let trimmed = formatted.trim_end_matches(".0");
        format!("{}{}", trimmed, unit)
    }
}

impl Deref for ByteSize {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<u64> for ByteSize {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

impl PartialEq<ByteSize> for u64 {
    fn eq(&self, other: &ByteSize) -> bool {
        *self == other.0
    }
}

impl fmt::Display for ByteSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_formatted())
    }
}

pub fn format_bytes(bytes: u64) -> String {
    ByteSize(bytes).to_string_formatted()
}
