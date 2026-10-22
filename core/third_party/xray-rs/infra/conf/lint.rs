// Module: infra\conf\lint.rs
// 1:1 Rust implementation corresponding to Go infra\conf\lint.go

use super::Config;
use crate::common::errors::{Error, Result};
use std::sync::RwLock;

pub trait ConfigureFilePostProcessingStage: Send + Sync {
    fn process(&self, conf: &mut Config) -> Result<()>;
}

static STAGES: RwLock<Vec<(&'static str, Box<dyn ConfigureFilePostProcessingStage>)>> =
    RwLock::new(Vec::new());

pub fn register_configure_file_post_processing_stage(
    name: &'static str,
    stage: Box<dyn ConfigureFilePostProcessingStage>,
) {
    if let Ok(mut list) = STAGES.write() {
        list.push((name, stage));
    }
}

pub fn post_process_configure_file(conf: &mut Config) -> Result<()> {
    if let Ok(list) = STAGES.read() {
        for (name, stage) in list.iter() {
            stage.process(conf).map_err(|e| {
                Error::Config(format!("Rejected by Postprocessing Stage {}: {}", name, e))
            })?;
        }
    }
    Ok(())
}

pub fn lint_config_json(json_str: &str) -> Vec<String> {
    let mut warnings = Vec::new();
    if json_str.contains("v2ray") {
        warnings.push("Deprecated keyword 'v2ray' found; use 'xray' instead".into());
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyStage;
    impl ConfigureFilePostProcessingStage for DummyStage {
        fn process(&self, conf: &mut Config) -> Result<()> {
            if conf.inbounds.is_empty() {
                // Pass
            }
            Ok(())
        }
    }

    #[test]
    fn test_lint_and_post_process() {
        let warnings = lint_config_json(r#"{"v2ray": true}"#);
        assert_eq!(warnings.len(), 1);

        register_configure_file_post_processing_stage("Dummy", Box::new(DummyStage));
        let mut cfg = Config::default();
        assert!(post_process_configure_file(&mut cfg).is_ok());
    }
}
