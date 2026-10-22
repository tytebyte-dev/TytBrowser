// Module: common\net\destination.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Endpoint {
    pub network: i32,
    pub address: Option<String>,
    pub port: u32,
}
