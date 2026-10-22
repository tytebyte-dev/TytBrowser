// Module: app\log\log.rs
// 1:1 Rust implementation corresponding to Go app\log\log.go

use super::LogManager;
use crate::common::errors::{Error, Result};

pub type Instance = LogManager;

pub fn parse_mask_address(c: &str) -> Result<(u8, u8)> {
    match c.to_lowercase().as_str() {
        "half" => Ok((16, 32)),
        "quarter" => Ok((8, 16)),
        "full" => Ok((0, 0)),
        "" => Ok((32, 128)),
        custom => {
            let parts: Vec<&str> = custom.split('+').collect();
            let mut m4 = 32u8;
            let mut m6 = 128u8;

            if let Some(p4) = parts.first() {
                let trimmed = p4.trim().trim_start_matches('/');
                if !trimmed.is_empty() {
                    m4 = trimmed.parse().map_err(|e| {
                        Error::Config(format!("Invalid IPv4 mask in mask_address: {}", e))
                    })?;
                }
            }
            if parts.len() >= 2 {
                let trimmed = parts[1].trim().trim_start_matches('/');
                if !trimmed.is_empty() {
                    m6 = trimmed.parse().map_err(|e| {
                        Error::Config(format!("Invalid IPv6 mask in mask_address: {}", e))
                    })?;
                }
            }

            if !m4.is_multiple_of(8) || m4 > 32 {
                return Err(Error::Config(
                    "Log Mask: ipv4 mask must be divisible by 8 and between 0-32".into(),
                ));
            }
            if m6 > 128 {
                return Err(Error::Config(
                    "Log Mask: ipv6 mask must be between 0-128".into(),
                ));
            }

            Ok((m4, m6))
        }
    }
}

pub struct MaskedMsgWrapper {
    pub mask4: u8,
    pub mask6: u8,
}

impl MaskedMsgWrapper {
    pub fn new(mask4: u8, mask6: u8) -> Self {
        Self { mask4, mask6 }
    }

    pub fn mask_ipv4_string(&self, s: &str) -> String {
        if self.mask4 == 32 {
            return s.to_string();
        }
        if self.mask4 == 0 {
            return "[Masked IPv4]".to_string();
        }
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 4 {
            return s.to_string();
        }
        let keep_parts = (self.mask4 / 8) as usize;
        let mut out = Vec::new();
        for (i, p) in parts.iter().enumerate() {
            if i < keep_parts {
                out.push(*p);
            } else {
                out.push("*");
            }
        }
        out.join(".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mask_address_presets() {
        assert_eq!(parse_mask_address("half").unwrap(), (16, 32));
        assert_eq!(parse_mask_address("quarter").unwrap(), (8, 16));
        assert_eq!(parse_mask_address("full").unwrap(), (0, 0));
        assert_eq!(parse_mask_address("").unwrap(), (32, 128));
    }

    #[test]
    fn test_parse_mask_address_custom() {
        assert_eq!(parse_mask_address("/16+/64").unwrap(), (16, 64));
        assert_eq!(parse_mask_address("/24").unwrap(), (24, 128));
        assert!(parse_mask_address("/15").is_err()); // not divisible by 8
    }

    #[test]
    fn test_masked_msg_wrapper() {
        let wrapper = MaskedMsgWrapper::new(16, 32);
        assert_eq!(wrapper.mask_ipv4_string("192.168.1.100"), "192.168.*.*");

        let full_mask = MaskedMsgWrapper::new(0, 0);
        assert_eq!(full_mask.mask_ipv4_string("1.2.3.4"), "[Masked IPv4]");

        let no_mask = MaskedMsgWrapper::new(32, 128);
        assert_eq!(no_mask.mask_ipv4_string("1.2.3.4"), "1.2.3.4");
    }
}
