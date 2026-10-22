// Module: app\stats\stats.rs
// 1:1 Rust implementation corresponding to Go app\stats\stats.go

use super::StatsManager;
use super::counter::Counter;
use std::sync::Arc;

pub const STATS_INBOUND_UPLINK: &str = "inbound>>>uplink";
pub const STATS_INBOUND_DOWNLINK: &str = "inbound>>>downlink";
pub const STATS_OUTBOUND_UPLINK: &str = "outbound>>>uplink";
pub const STATS_OUTBOUND_DOWNLINK: &str = "outbound>>>downlink";

pub fn inbound_uplink_name(tag: &str) -> String {
    format!("inbound>>>{tag}>>>traffic>>>uplink")
}

pub fn inbound_downlink_name(tag: &str) -> String {
    format!("inbound>>>{tag}>>>traffic>>>downlink")
}

pub fn outbound_uplink_name(tag: &str) -> String {
    format!("outbound>>>{tag}>>>traffic>>>uplink")
}

pub fn outbound_downlink_name(tag: &str) -> String {
    format!("outbound>>>{tag}>>>traffic>>>downlink")
}

pub fn user_uplink_name(email: &str) -> String {
    format!("user>>>{email}>>>traffic>>>uplink")
}

pub fn user_downlink_name(email: &str) -> String {
    format!("user>>>{email}>>>traffic>>>downlink")
}

pub fn get_or_register_counter(mgr: &StatsManager, name: &str) -> Arc<Counter> {
    mgr.register_counter_sync(name)
}
