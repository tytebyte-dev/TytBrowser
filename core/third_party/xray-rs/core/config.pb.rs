// Module: core\config.pb.rs
// 1:1 Rust protobuf message definitions corresponding to Go core\config.pb.go

use crate::common::serial::TypedMessage;
use serde::{Deserialize, Serialize};

/// InboundHandlerConfig is the configuration for inbound handler.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InboundHandlerConfig {
    #[serde(default)]
    pub tag: String,
    #[serde(rename = "receiver_settings", alias = "receiverSettings", default)]
    pub receiver_settings: Option<TypedMessage>,
    #[serde(rename = "proxy_settings", alias = "proxySettings", default)]
    pub proxy_settings: Option<TypedMessage>,
}

impl InboundHandlerConfig {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            receiver_settings: None,
            proxy_settings: None,
        }
    }

    pub fn get_tag(&self) -> &str {
        &self.tag
    }

    pub fn get_receiver_settings(&self) -> Option<&TypedMessage> {
        self.receiver_settings.as_ref()
    }

    pub fn get_proxy_settings(&self) -> Option<&TypedMessage> {
        self.proxy_settings.as_ref()
    }
}

/// OutboundHandlerConfig is the configuration for outbound handler.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutboundHandlerConfig {
    #[serde(default)]
    pub tag: String,
    #[serde(rename = "sender_settings", alias = "senderSettings", default)]
    pub sender_settings: Option<TypedMessage>,
    #[serde(rename = "proxy_settings", alias = "proxySettings", default)]
    pub proxy_settings: Option<TypedMessage>,
    #[serde(default)]
    pub expire: i64,
    #[serde(default)]
    pub comment: String,
}

impl OutboundHandlerConfig {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            sender_settings: None,
            proxy_settings: None,
            expire: 0,
            comment: String::new(),
        }
    }

    pub fn get_tag(&self) -> &str {
        &self.tag
    }

    pub fn get_sender_settings(&self) -> Option<&TypedMessage> {
        self.sender_settings.as_ref()
    }

    pub fn get_proxy_settings(&self) -> Option<&TypedMessage> {
        self.proxy_settings.as_ref()
    }
}

/// Config is the master config of Xray.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub inbound: Vec<InboundHandlerConfig>,
    #[serde(default)]
    pub outbound: Vec<OutboundHandlerConfig>,
    #[serde(default)]
    pub app: Vec<TypedMessage>,
    #[serde(default)]
    pub extension: Vec<TypedMessage>,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_inbound(&self) -> &[InboundHandlerConfig] {
        &self.inbound
    }

    pub fn get_outbound(&self) -> &[OutboundHandlerConfig] {
        &self.outbound
    }

    pub fn get_app(&self) -> &[TypedMessage] {
        &self.app
    }

    pub fn get_extension(&self) -> &[TypedMessage] {
        &self.extension
    }
}
