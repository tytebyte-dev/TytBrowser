#[path = "config.pb.rs"]
pub mod config_pb;
pub mod version;

use crate::common::errors::{Error, Result};
pub use config_pb::Config as VersionConfig;
pub use version::{VERSION, Version};

pub fn compare_versions(v1: &str, v2: &str) -> Result<i32> {
    let mut parts1: Vec<u32> = v1
        .split('.')
        .map(|s| {
            s.parse::<u32>()
                .map_err(|_| Error::Config(format!("Invalid version segment: {}", s)))
        })
        .collect::<Result<Vec<_>>>()?;

    let mut parts2: Vec<u32> = v2
        .split('.')
        .map(|s| {
            s.parse::<u32>()
                .map_err(|_| Error::Config(format!("Invalid version segment: {}", s)))
        })
        .collect::<Result<Vec<_>>>()?;

    while parts1.len() < parts2.len() {
        parts1.push(0);
    }
    while parts2.len() < parts1.len() {
        parts2.push(0);
    }

    for (p1, p2) in parts1.iter().zip(parts2.iter()) {
        if p1 < p2 {
            return Ok(-1);
        } else if p1 > p2 {
            return Ok(1);
        }
    }

    Ok(0)
}

pub fn validate_version(core_version: &str, min_version: &str, max_version: &str) -> Result<()> {
    if !min_version.is_empty() {
        let cmp = compare_versions(min_version, core_version)?;
        if cmp > 0 {
            return Err(Error::Config(format!(
                "Config requires Xray version {} or higher, current version is {}",
                min_version, core_version
            )));
        }
    }

    if !max_version.is_empty() {
        let cmp = compare_versions(max_version, core_version)?;
        if cmp < 0 {
            return Err(Error::Config(format!(
                "Config requires Xray version {} or lower, current version is {}",
                max_version, core_version
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compare_and_validate() {
        assert_eq!(compare_versions("1.8.0", "1.8.0").unwrap(), 0);
        assert_eq!(compare_versions("1.8.1", "1.8.0").unwrap(), 1);
        assert_eq!(compare_versions("1.7.0", "1.8.0").unwrap(), -1);

        assert!(validate_version("1.8.4", "1.8.0", "1.9.0").is_ok());
        assert!(validate_version("1.7.9", "1.8.0", "1.9.0").is_err());
        assert!(validate_version("1.9.1", "1.8.0", "1.9.0").is_err());
    }

    #[test]
    fn test_version_config_pb() {
        let cfg = VersionConfig {
            core_version: "26.3.27".into(),
            min_version: "25.0.0".into(),
            max_version: "27.0.0".into(),
        };
        assert_eq!(cfg.core_version, "26.3.27");
        assert!(validate_version(&cfg.core_version, &cfg.min_version, &cfg.max_version).is_ok());

        let ver = Version::new(cfg).unwrap();
        assert_eq!(ver.config.core_version, "26.3.27");

        let bad_cfg = VersionConfig {
            core_version: "24.0.0".into(),
            min_version: "25.0.0".into(),
            max_version: "".into(),
        };
        assert!(Version::new(bad_cfg).is_err());
    }
}
