// Module: app\stats\command\command.pb.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetStatsRequest {
    pub name: String,
    pub reset: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stat {
    pub name: String,
    pub value: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetStatsResponse {
    pub stat: Option<Stat>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryStatsRequest {
    pub pattern: String,
    pub reset: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryStatsResponse {
    pub stat: Vec<Stat>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetSysStatsRequest {}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SysStatsResponse {
    pub num_goroutine: u32,
    pub num_gc: u32,
    pub alloc: u64,
    pub total_alloc: u64,
    pub sys: u64,
    pub mallocs: u64,
    pub frees: u64,
    pub live_objects: u64,
    pub pause_total_ns: u64,
    pub uptime: u32,
}
