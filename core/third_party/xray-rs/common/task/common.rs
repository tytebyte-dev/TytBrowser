// Module: common\task\common.rs
// 1:1 Rust implementation corresponding to Go common\task\common.go

use crate::common::common::Closable;
use crate::common::errors::Result;

pub const DEFAULT_TASK_TIMEOUT_SECS: u64 = 60;

/// Close returns a closure that closes v.
pub fn close_task<C: Closable>(closable: C) -> impl Fn() -> Result<()> {
    move || closable.close()
}
