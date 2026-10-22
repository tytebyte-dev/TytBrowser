// Module: transport\internet\headers\http\config.pb.rs
// 1:1 Rust implementation corresponding to Go transport\internet\headers\http\config.pb.go

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub value: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Method {
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestConfig {
    #[serde(default)]
    pub version: Option<Version>,
    #[serde(default)]
    pub method: Option<Method>,
    #[serde(default)]
    pub uri: Vec<String>,
    #[serde(default)]
    pub header: Vec<Header>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseConfig {
    #[serde(default)]
    pub version: Option<Version>,
    #[serde(default)]
    pub status: Option<Status>,
    #[serde(default)]
    pub header: Vec<Header>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub request: Option<RequestConfig>,
    #[serde(default)]
    pub response: Option<ResponseConfig>,
}
