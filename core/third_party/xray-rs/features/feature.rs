// Module: features\feature.rs
// 1:1 Rust implementation corresponding to Go features\feature.go

use crate::common::errors::Result;

pub const TYPE_DNS_CLIENT: &str = "dns_client";
pub const TYPE_INBOUND_MANAGER: &str = "inbound_manager";
pub const TYPE_OUTBOUND_MANAGER: &str = "outbound_manager";
pub const TYPE_POLICY_MANAGER: &str = "policy_manager";
pub const TYPE_ROUTER: &str = "router";
pub const TYPE_DISPATCHER: &str = "dispatcher";
pub const TYPE_STATS_MANAGER: &str = "stats_manager";
pub const TYPE_OBSERVATORY: &str = "observatory";
pub const TYPE_FAKE_DNS: &str = "fake_dns";

pub trait Feature: Send + Sync {
    fn feature_type(&self) -> &'static str;

    fn start(&self) -> Result<()> {
        Ok(())
    }

    fn close(&self) -> Result<()> {
        Ok(())
    }
}
