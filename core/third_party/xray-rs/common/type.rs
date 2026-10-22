// Module: common\type.rs
// 1:1 Rust implementation corresponding to Go common\type.go

use crate::common::errors::{Error, Result};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::RwLock;

pub type ConfigCreator = Box<dyn Fn(Box<dyn Any>) -> Result<Box<dyn Any>> + Send + Sync>;

lazy_static::lazy_static! {
    static ref TYPE_CREATOR_REGISTRY: RwLock<HashMap<TypeId, ConfigCreator>> = RwLock::new(HashMap::new());
}

pub fn type_of<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}

pub fn register_config<T: 'static, F>(creator: F) -> Result<()>
where
    F: Fn(T) -> Result<Box<dyn Any>> + Send + Sync + 'static,
{
    let type_id = TypeId::of::<T>();
    let mut guard = TYPE_CREATOR_REGISTRY
        .write()
        .map_err(|_| Error::Other("Config registry lock poisoned".into()))?;

    if guard.contains_key(&type_id) {
        return Err(Error::Other(format!(
            "Type {} is already registered",
            std::any::type_name::<T>()
        )));
    }

    guard.insert(
        type_id,
        Box::new(move |boxed_any| {
            let val = boxed_any
                .downcast::<T>()
                .map_err(|_| Error::Other("Failed to downcast config object".into()))?;
            creator(*val)
        }),
    );
    Ok(())
}

pub fn create_object<T: 'static>(config: T) -> Result<Box<dyn Any>> {
    let type_id = TypeId::of::<T>();
    let guard = TYPE_CREATOR_REGISTRY
        .read()
        .map_err(|_| Error::Other("Config registry lock poisoned".into()))?;

    let creator = guard.get(&type_id).ok_or_else(|| {
        Error::NotFound(format!(
            "Config type {} is not registered",
            std::any::type_name::<T>()
        ))
    })?;

    creator(Box::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SampleConfig {
        name: String,
    }

    struct SampleObject {
        greeting: String,
    }

    #[test]
    fn test_type_registration_and_object_creation() {
        assert_eq!(type_of(&42), "i32");

        let _ = register_config::<SampleConfig, _>(|cfg| {
            Ok(Box::new(SampleObject {
                greeting: format!("Hello, {}!", cfg.name),
            }))
        });

        let obj = create_object(SampleConfig {
            name: "Xray".into(),
        })
        .unwrap();

        let sample = obj.downcast::<SampleObject>().unwrap();
        assert_eq!(sample.greeting, "Hello, Xray!");
    }
}
