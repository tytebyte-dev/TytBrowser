// Module: common\utils\access_field.rs
// 1:1 Rust implementation corresponding to Go common\utils\access_field.go

use std::any::Any;
use std::collections::HashMap;

/// Trait providing dynamic reflection-style field access for structs.
pub trait ReflectFields {
    fn get_field_ref<T: 'static>(&self, field_name: &str) -> Option<&T>;
    fn get_field_mut<T: 'static>(&mut self, field_name: &str) -> Option<&mut T>;
}

/// DynamicFieldHolder can be embedded in or used alongside structs to store
/// unexported / dynamic fields accessed by name at runtime.
#[derive(Default)]
pub struct DynamicFieldHolder {
    fields: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl DynamicFieldHolder {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    pub fn set_field<T: 'static + Send + Sync>(&mut self, name: &str, value: T) {
        self.fields.insert(name.to_string(), Box::new(value));
    }

    /// AccessField corresponding to Go AccessField[valueType](obj, fieldName) *valueType
    /// Returns a reference to the field or panics if field is missing or type mismatched.
    pub fn access_field<T: 'static>(&self, field_name: &str) -> &T {
        match self.fields.get(field_name) {
            Some(boxed) => match boxed.downcast_ref::<T>() {
                Some(val) => val,
                None => panic!(
                    "field type mismatch for field '{}': expected {}",
                    field_name,
                    std::any::type_name::<T>()
                ),
            },
            None => panic!("field '{}' not found", field_name),
        }
    }

    /// Mutable access corresponding to pointer dereference in Go
    pub fn access_field_mut<T: 'static>(&mut self, field_name: &str) -> &mut T {
        match self.fields.get_mut(field_name) {
            Some(boxed) => match boxed.downcast_mut::<T>() {
                Some(val) => val,
                None => panic!(
                    "field type mismatch for field '{}': expected {}",
                    field_name,
                    std::any::type_name::<T>()
                ),
            },
            None => panic!("field '{}' not found", field_name),
        }
    }

    pub fn try_access_field<T: 'static>(&self, field_name: &str) -> Option<&T> {
        self.fields
            .get(field_name)
            .and_then(|b| b.downcast_ref::<T>())
    }
}

pub fn get_field_or_default<'a>(val: Option<&'a str>, default: &'a str) -> &'a str {
    val.unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_field_access() {
        let mut holder = DynamicFieldHolder::new();
        holder.set_field("timeout", 30u32);
        holder.set_field("tag", "proxy_inbound".to_string());

        let timeout: &u32 = holder.access_field("timeout");
        assert_eq!(*timeout, 30);

        let tag: &String = holder.access_field("tag");
        assert_eq!(tag, "proxy_inbound");

        // Mutate field
        let timeout_mut: &mut u32 = holder.access_field_mut("timeout");
        *timeout_mut = 60;
        assert_eq!(*holder.access_field::<u32>("timeout"), 60);

        assert_eq!(get_field_or_default(Some("custom"), "default"), "custom");
        assert_eq!(get_field_or_default(None, "default"), "default");
    }

    #[test]
    #[should_panic(expected = "field 'missing' not found")]
    fn test_dynamic_field_missing() {
        let holder = DynamicFieldHolder::new();
        let _: &u32 = holder.access_field("missing");
    }

    #[test]
    #[should_panic(expected = "field type mismatch")]
    fn test_dynamic_field_type_mismatch() {
        let mut holder = DynamicFieldHolder::new();
        holder.set_field("count", 100u64);
        let _: &u32 = holder.access_field("count");
    }
}
