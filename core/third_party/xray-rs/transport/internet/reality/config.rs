use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RealityConfig {
    #[serde(default, rename = "show")]
    pub show: bool,
    #[serde(default, rename = "dest")]
    pub dest: Option<String>,
    #[serde(default, rename = "serverNames")]
    pub server_names: Vec<String>,
    #[serde(default, rename = "privateKey")]
    pub private_key: Option<String>,
    #[serde(default, rename = "publicKey")]
    pub public_key: Option<String>,
    #[serde(default, rename = "minClientVer")]
    pub min_client_ver: Option<String>,
    #[serde(default, rename = "maxClientVer")]
    pub max_client_ver: Option<String>,
    #[serde(default, rename = "maxTimeDiff")]
    pub max_time_diff: Option<u64>,
    #[serde(default, rename = "shortIds")]
    pub short_ids: Vec<String>,
    #[serde(default, rename = "spiderX")]
    pub spider_x: Option<String>,
}
