// Module: common\serial\typed_message.rs
// 1:1 Rust implementation corresponding to Go common\serial\typed_message.go

use crate::common::errors::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedMessage {
    #[serde(rename = "type", default)]
    pub r#type: String,
    #[serde(default)]
    pub value: Vec<u8>,
}

impl TypedMessage {
    pub fn new(type_name: impl Into<String>, value: Vec<u8>) -> Self {
        Self {
            r#type: type_name.into(),
            value,
        }
    }

    pub fn type_name(&self) -> &str {
        &self.r#type
    }

    pub fn get_instance(&self) -> Result<&[u8]> {
        if self.r#type.is_empty() {
            return Err(Error::Config("empty message type".to_string()));
        }
        Ok(&self.value)
    }
}

/// ToTypedMessage converts an optional proto message / byte pair into TypedMessage.
/// Returns None if message is None.
pub fn to_typed_message(message: Option<(&str, Vec<u8>)>) -> Option<TypedMessage> {
    message.map(|(type_name, value)| TypedMessage::new(type_name, value))
}

/// GetInstance creates / resolves a message type or validates it.
pub fn get_instance(message_type: &str) -> Result<Option<TypedMessage>> {
    if message_type.is_empty() {
        return Err(Error::Config("empty message type".to_string()));
    }
    Ok(None)
}
