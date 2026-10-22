// Module: transport\internet\splithttp\config.pb.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangeConfig {
    #[serde(default)]
    pub from: i32,
    #[serde(default)]
    pub to: i32,
}

impl RangeConfig {
    pub fn new(from: i32, to: i32) -> Self {
        Self { from, to }
    }

    pub fn rand(&self) -> i32 {
        if self.from >= self.to {
            return self.from;
        }
        use rand::Rng;
        rand::thread_rng().gen_range(self.from..=self.to)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct XmuxConfig {
    pub max_concurrency: Option<RangeConfig>,
    pub max_connections: Option<RangeConfig>,
    pub c_max_reuse_times: Option<RangeConfig>,
    pub h_max_request_times: Option<RangeConfig>,
    pub h_max_reusable_secs: Option<RangeConfig>,
    #[serde(default)]
    pub h_keep_alive_period: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    pub x_padding_bytes: Option<RangeConfig>,
    #[serde(default)]
    pub no_grpc_header: bool,
    #[serde(default)]
    pub no_sse_header: bool,
    pub sc_max_each_post_bytes: Option<RangeConfig>,
    pub sc_min_posts_interval_ms: Option<RangeConfig>,
    #[serde(default)]
    pub sc_max_buffered_posts: i64,
    pub sc_stream_up_server_secs: Option<RangeConfig>,
    pub xmux: Option<XmuxConfig>,
    #[serde(default)]
    pub x_padding_obfs_mode: bool,
    #[serde(default)]
    pub x_padding_key: String,
    #[serde(default)]
    pub x_padding_header: String,
    #[serde(default)]
    pub x_padding_placement: String,
    #[serde(default)]
    pub x_padding_method: String,
    #[serde(default)]
    pub uplink_http_method: String,
    #[serde(default)]
    pub session_placement: String,
    #[serde(default)]
    pub session_key: String,
    #[serde(default)]
    pub seq_placement: String,
    #[serde(default)]
    pub seq_key: String,
    #[serde(default)]
    pub uplink_data_placement: String,
    #[serde(default)]
    pub uplink_data_key: String,
    pub uplink_chunk_size: Option<RangeConfig>,
    #[serde(default)]
    pub server_max_header_bytes: i32,
}
