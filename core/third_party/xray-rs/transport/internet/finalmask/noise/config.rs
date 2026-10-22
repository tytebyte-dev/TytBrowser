// Module: transport\internet\finalmask\noise\config.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\noise\config.go & config.proto

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoiseItem {
    #[serde(default)]
    pub rand_min: i64,
    #[serde(default)]
    pub rand_max: i64,
    #[serde(default)]
    pub rand_range_min: i32,
    #[serde(default = "default_rand_range_max")]
    pub rand_range_max: i32,
    #[serde(default)]
    pub packet: Vec<u8>,
    #[serde(default)]
    pub delay_min: i64,
    #[serde(default)]
    pub delay_max: i64,
}

fn default_rand_range_max() -> i32 {
    255
}

impl Default for NoiseItem {
    fn default() -> Self {
        Self {
            rand_min: 0,
            rand_max: 0,
            rand_range_min: 0,
            rand_range_max: 255,
            packet: Vec::new(),
            delay_min: 0,
            delay_max: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoiseConfig {
    #[serde(default)]
    pub reset_min: i64,
    #[serde(default)]
    pub reset_max: i64,
    #[serde(default)]
    pub items: Vec<NoiseItem>,
    #[serde(default)]
    pub length: String,
    #[serde(default)]
    pub delay: String,
    #[serde(default)]
    pub str_pattern: Option<String>,
    // Compatibility fields
    #[serde(default)]
    pub min_len: usize,
    #[serde(default)]
    pub max_len: usize,
    #[serde(default)]
    pub min_delay: Duration,
    #[serde(default)]
    pub max_delay: Duration,
}

impl Default for NoiseConfig {
    fn default() -> Self {
        Self {
            reset_min: 0,
            reset_max: 0,
            items: Vec::new(),
            length: String::new(),
            delay: String::new(),
            str_pattern: None,
            min_len: 10,
            max_len: 20,
            min_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(16),
        }
    }
}

impl NoiseConfig {
    pub fn new(length: impl Into<String>, delay: impl Into<String>) -> Self {
        Self {
            length: length.into(),
            delay: delay.into(),
            ..Default::default()
        }
    }
}
