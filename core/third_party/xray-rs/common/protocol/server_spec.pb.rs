// Module: common\protocol\server_spec.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerEndpoint {
    pub address: Option<String>,
    pub port: u32,
}
