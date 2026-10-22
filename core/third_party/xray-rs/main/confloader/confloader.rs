// Module: main\confloader\confloader.rs
// 1:1 Rust implementation corresponding to Go main\confloader\confloader.go

use crate::common::errors::{Error, Result};
use crate::infra::conf::Config;
use std::sync::RwLock;

pub type ConfigFileLoader = fn(&str) -> Result<Vec<u8>>;

static EFFECTIVE_LOADER: RwLock<Option<ConfigFileLoader>> = RwLock::new(None);

pub fn set_effective_config_file_loader(loader: ConfigFileLoader) {
    if let Ok(mut guard) = EFFECTIVE_LOADER.write() {
        *guard = Some(loader);
    }
}

pub fn load_config(file: &str) -> Result<Vec<u8>> {
    let custom_loader = EFFECTIVE_LOADER.read().ok().and_then(|g| *g);
    if let Some(loader) = custom_loader {
        loader(file)
    } else {
        if file.is_empty() || file == "-" || file == "stdin:" {
            use std::io::Read;
            let mut buf = Vec::new();
            std::io::stdin().read_to_end(&mut buf).map_err(Error::Io)?;
            return Ok(buf);
        }
        std::fs::read(file).map_err(Error::Io)
    }
}

pub fn load_config_from_str(content: &str) -> Result<Config> {
    serde_json::from_str(content).map_err(|e| Error::Config(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confloader_load_from_str() {
        let json_str = r#"{"log": {"loglevel": "warning"}}"#;
        let cfg = load_config_from_str(json_str).unwrap();
        assert_eq!(
            cfg.log.as_ref().unwrap().loglevel.as_deref(),
            Some("warning")
        );
    }

    #[test]
    fn test_confloader_custom_loader() {
        fn dummy_loader(path: &str) -> Result<Vec<u8>> {
            Ok(format!(r#"{{"tag": "{}"}}"#, path).into_bytes())
        }

        set_effective_config_file_loader(dummy_loader);
        let res = load_config("custom-test.json").unwrap();
        assert_eq!(res, br#"{"tag": "custom-test.json"}"#);
    }
}
