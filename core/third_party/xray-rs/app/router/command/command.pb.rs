// Module: app\router\command\command.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingContext {
    pub inbound_tag: String,
    pub network: i32,
    pub source_ip_s: Vec<Vec<u8>>,
    pub target_ip_s: Vec<Vec<u8>>,
    pub source_port: u32,
    pub target_port: u32,
    pub target_domain: String,
    pub protocol: String,
    pub user: String,
    pub attributes: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscribeRoutingStatsRequest {
    pub field_selectors: Vec<String>,
}
