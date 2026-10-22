// Module: transport\internet\finalmask\fragment\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub packets_from: i64,
    #[serde(default)]
    pub packets_to: i64,
    #[serde(default)]
    pub length_min: i64,
    #[serde(default)]
    pub length_max: i64,
    #[serde(default)]
    pub delay_min: i64,
    #[serde(default)]
    pub delay_max: i64,
    #[serde(default)]
    pub max_split_min: i64,
    #[serde(default)]
    pub max_split_max: i64,
}
