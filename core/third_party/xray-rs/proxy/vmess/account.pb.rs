// Module: proxy\vmess\account.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub alter_id: u32,
    pub security_settings: Option<String>,
    pub authenticated_length: bool,
}
