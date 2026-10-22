// Module: transport\internet\splithttp\config.rs
// 1:1 Rust implementation corresponding to Go transport\internet\splithttp\config.go

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::common::*;

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
pub struct SplitHttpConfig {
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
    #[serde(default)]
    pub max_upload_size: usize,
    #[serde(default)]
    pub max_concurrent_uploads: usize,
}

pub type Config = SplitHttpConfig;

impl SplitHttpConfig {
    pub fn new(path: impl Into<String>, host: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            host: host.into(),
            headers: HashMap::new(),
            max_upload_size: 1024 * 1024,
            max_concurrent_uploads: 8,
            ..Default::default()
        }
    }

    pub fn get_normalized_path(&self) -> String {
        let parts: Vec<&str> = self.path.splitn(2, '?').collect();
        let mut path = parts[0].to_string();
        if path.is_empty() || !path.starts_with('/') {
            path = format!("/{}", path);
        }
        if !path.ends_with('/') {
            path.push('/');
        }
        path
    }

    pub fn get_normalized_query(&self) -> String {
        let parts: Vec<&str> = self.path.splitn(2, '?').collect();
        if parts.len() > 1 {
            parts[1].to_string()
        } else {
            String::new()
        }
    }

    pub fn get_normalized_uplink_http_method(&self) -> &str {
        if self.uplink_http_method.is_empty() {
            "POST"
        } else {
            &self.uplink_http_method
        }
    }

    pub fn get_normalized_sc_max_each_post_bytes(&self) -> RangeConfig {
        match self.sc_max_each_post_bytes {
            Some(rc) if rc.to > 0 => rc,
            _ => RangeConfig::new(1_000_000, 1_000_000),
        }
    }

    pub fn get_normalized_sc_min_posts_interval_ms(&self) -> RangeConfig {
        match self.sc_min_posts_interval_ms {
            Some(rc) if rc.to > 0 => rc,
            _ => RangeConfig::new(30, 30),
        }
    }

    pub fn get_normalized_sc_max_buffered_posts(&self) -> usize {
        if self.sc_max_buffered_posts <= 0 {
            30
        } else {
            self.sc_max_buffered_posts as usize
        }
    }

    pub fn get_normalized_sc_stream_up_server_secs(&self) -> RangeConfig {
        match self.sc_stream_up_server_secs {
            Some(rc) if rc.to > 0 => rc,
            _ => RangeConfig::new(20, 80),
        }
    }

    pub fn get_normalized_session_placement(&self) -> &str {
        if self.session_placement.is_empty() {
            PLACEMENT_PATH
        } else {
            &self.session_placement
        }
    }

    pub fn get_normalized_seq_placement(&self) -> &str {
        if self.seq_placement.is_empty() {
            PLACEMENT_PATH
        } else {
            &self.seq_placement
        }
    }

    pub fn get_normalized_uplink_data_placement(&self) -> &str {
        if self.uplink_data_placement.is_empty() {
            PLACEMENT_BODY
        } else {
            &self.uplink_data_placement
        }
    }

    pub fn get_normalized_session_key(&self) -> &str {
        if !self.session_key.is_empty() {
            return &self.session_key;
        }
        match self.get_normalized_session_placement() {
            PLACEMENT_HEADER => "X-Session",
            PLACEMENT_COOKIE | PLACEMENT_QUERY => "x_session",
            _ => "",
        }
    }

    pub fn get_normalized_seq_key(&self) -> &str {
        if !self.seq_key.is_empty() {
            return &self.seq_key;
        }
        match self.get_normalized_seq_placement() {
            PLACEMENT_HEADER => "X-Seq",
            PLACEMENT_COOKIE | PLACEMENT_QUERY => "x_seq",
            _ => "",
        }
    }

    pub fn get_normalized_server_max_header_bytes(&self) -> usize {
        if self.server_max_header_bytes <= 0 {
            8192
        } else {
            self.server_max_header_bytes as usize
        }
    }

    pub fn append_to_path(path: &str, value: &str) -> String {
        if path.ends_with('/') {
            format!("{}{}", path, value)
        } else {
            format!("{}/{}", path, value)
        }
    }

    pub fn extract_meta_from_request(
        &self,
        req_path: &str,
        base_path: &str,
        headers: &HashMap<String, String>,
        query_pairs: &HashMap<String, String>,
        cookies: &HashMap<String, String>,
    ) -> (String, String) {
        let session_placement = self.get_normalized_session_placement();
        let seq_placement = self.get_normalized_seq_placement();
        let session_key = self.get_normalized_session_key();
        let seq_key = self.get_normalized_seq_key();

        let mut subpath: Vec<&str> = Vec::new();
        if (session_placement == PLACEMENT_PATH || seq_placement == PLACEMENT_PATH)
            && req_path.starts_with(base_path)
        {
            let rem = &req_path[base_path.len()..];
            subpath = rem.split('/').filter(|s| !s.is_empty()).collect();
        }

        let mut path_part = 0;
        let mut session_id = String::new();
        match session_placement {
            PLACEMENT_PATH => {
                if subpath.len() > path_part {
                    session_id = subpath[path_part].to_string();
                    path_part += 1;
                }
            }
            PLACEMENT_QUERY => {
                if let Some(val) = query_pairs.get(session_key) {
                    session_id = val.clone();
                }
            }
            PLACEMENT_HEADER => {
                if let Some(val) = headers.get(session_key) {
                    session_id = val.clone();
                }
            }
            PLACEMENT_COOKIE => {
                if let Some(val) = cookies.get(session_key) {
                    session_id = val.clone();
                }
            }
            _ => {}
        }

        let mut seq_str = String::new();
        match seq_placement {
            PLACEMENT_PATH => {
                if subpath.len() > path_part {
                    seq_str = subpath[path_part].to_string();
                }
            }
            PLACEMENT_QUERY => {
                if let Some(val) = query_pairs.get(seq_key) {
                    seq_str = val.clone();
                }
            }
            PLACEMENT_HEADER => {
                if let Some(val) = headers.get(seq_key) {
                    seq_str = val.clone();
                }
            }
            PLACEMENT_COOKIE => {
                if let Some(val) = cookies.get(seq_key) {
                    seq_str = val.clone();
                }
            }
            _ => {}
        }

        (session_id, seq_str)
    }
}
