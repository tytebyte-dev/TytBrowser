// Module: infra\conf\loader.rs
// 1:1 Rust implementation corresponding to Go infra\conf\loader.go

use super::Config;
use crate::common::errors::{Error, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub type ConfigCreator = Arc<dyn Fn() -> Value + Send + Sync>;

#[derive(Default, Clone)]
pub struct ConfigCreatorCache {
    creators: HashMap<String, ConfigCreator>,
}

impl ConfigCreatorCache {
    pub fn new() -> Self {
        Self {
            creators: HashMap::new(),
        }
    }

    pub fn register_creator(&mut self, id: &str, creator: ConfigCreator) -> Result<()> {
        if self.creators.contains_key(id) {
            return Err(Error::Config(format!("{} already registered", id)));
        }
        self.creators.insert(id.to_string(), creator);
        Ok(())
    }

    pub fn create_config(&self, id: &str) -> Result<Value> {
        match self.creators.get(id) {
            Some(creator) => Ok(creator()),
            None => Err(Error::NotFound(format!("unknown config id: {}", id))),
        }
    }
}

pub struct JSONConfigLoader {
    cache: ConfigCreatorCache,
    id_key: String,
    config_key: String,
}

impl JSONConfigLoader {
    pub fn new(cache: ConfigCreatorCache, id_key: &str, config_key: &str) -> Self {
        Self {
            cache,
            id_key: id_key.to_string(),
            config_key: config_key.to_string(),
        }
    }

    pub fn load_with_id(&self, raw: &[u8], id: &str) -> Result<Value> {
        let id_lower = id.to_lowercase();
        let _ = self.cache.create_config(&id_lower)?;
        let val: Value = serde_json::from_slice(raw)
            .map_err(|e| Error::Config(format!("JSON parse error: {}", e)))?;
        Ok(val)
    }

    pub fn load(&self, raw: &[u8]) -> Result<(Value, String)> {
        let obj: HashMap<String, Value> = serde_json::from_slice(raw)
            .map_err(|e| Error::Config(format!("JSON root parse error: {}", e)))?;

        let raw_id = obj
            .get(&self.id_key)
            .ok_or_else(|| Error::NotFound(format!("{} not found in JSON context", self.id_key)))?;

        let id = raw_id
            .as_str()
            .ok_or_else(|| Error::Config(format!("{} must be a string", self.id_key)))?
            .to_string();

        let raw_config = if !self.config_key.is_empty() {
            obj.get(&self.config_key)
                .cloned()
                .unwrap_or(Value::Object(serde_json::Map::new()))
        } else {
            Value::Object(obj.into_iter().collect())
        };

        let raw_bytes =
            serde_json::to_vec(&raw_config).map_err(|e| Error::Config(e.to_string()))?;

        let config = self.load_with_id(&raw_bytes, &id)?;
        Ok((config, id))
    }
}

pub fn load_config(json_str: &str) -> Result<Config> {
    serde_json::from_str(json_str).map_err(|e| Error::Config(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_config_loader() {
        let mut cache = ConfigCreatorCache::new();
        cache
            .register_creator(
                "freedom",
                Arc::new(|| serde_json::json!({"domainStrategy": "AsIs"})),
            )
            .unwrap();

        assert!(
            cache
                .register_creator("freedom", Arc::new(|| serde_json::json!({})))
                .is_err()
        );

        let loader = JSONConfigLoader::new(cache, "protocol", "settings");
        let raw = br#"{"protocol":"freedom","settings":{"domainStrategy":"UseIP"}}"#;
        let (val, id) = loader.load(raw).unwrap();
        assert_eq!(id, "freedom");
        assert_eq!(val["domainStrategy"], "UseIP");
    }
}
