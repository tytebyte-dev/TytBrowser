// Module: proxy\vless\account.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub flow: String,
    pub encryption: String,
}
