// Module: common\reflect\marshal.rs
// 1:1 Rust implementation corresponding to Go common\reflect\marshal.go

use crate::common::errors::{Error, Result};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

pub fn json_marshal_without_escape<T: Serialize>(t: &T) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    t.serialize(&mut ser)
        .map_err(|e| Error::Config(e.to_string()))?;
    buf.push(b'\n');
    Ok(buf)
}

pub fn marshal_to_json<T: Serialize>(v: &T, insert_type_info: bool) -> (String, bool) {
    if let Ok(mut val) = serde_json::to_value(v) {
        if insert_type_info && let Value::Object(ref mut map) = val {
            map.insert(
                "_TypedMessage_".to_string(),
                Value::String(std::any::type_name::<T>().to_string()),
            );
        }
        if let Ok(bytes) = json_marshal_without_escape(&val)
            && let Ok(s) = String::from_utf8(bytes)
        {
            return (s, true);
        }
    }
    (String::new(), false)
}

pub fn to_json<T: Serialize>(val: &T) -> Result<String> {
    serde_json::to_string(val).map_err(|e| Error::Config(e.to_string()))
}

pub fn from_json<T: DeserializeOwned>(s: &str) -> Result<T> {
    serde_json::from_str(s).map_err(|e| Error::Config(e.to_string()))
}
