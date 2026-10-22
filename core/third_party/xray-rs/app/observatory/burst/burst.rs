// Module: app\observatory\burst\burst.rs
// 1:1 Rust implementation corresponding to Go app\observatory\burst\burst.go

use std::time::Duration;

pub const RTT_FAILED: Duration = Duration::from_nanos(i64::MAX as u64);
pub const RTT_UNTESTED: Duration = Duration::from_nanos((i64::MAX - 1) as u64);
pub const RTT_UNQUALIFIED: Duration = Duration::from_nanos((i64::MAX - 2) as u64);

pub const MODULE_NAME: &str = "burst_observatory";
