// Module: transport\internet\reality\config.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub show: bool,
    pub dest: String,
    pub r#type: String,
    pub server_names: Vec<String>,
    pub private_key: Vec<u8>,
    pub min_client_ver: Vec<u8>,
    pub max_client_ver: Vec<u8>,
    pub max_time_diff: u64,
    pub short_ids: Vec<Vec<u8>>,
    pub fingerprint: String,
    pub server_name: String,
    pub public_key: Vec<u8>,
    pub short_id: Vec<u8>,
    pub spider_x: String,
}
