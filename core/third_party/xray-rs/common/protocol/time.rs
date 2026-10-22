use crate::common::dice::roll;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(pub i64);

impl Timestamp {
    pub fn now() -> Self {
        let sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        Timestamp(sec)
    }

    pub fn with_jitter(&self, delta: i64) -> Self {
        if delta <= 0 {
            return *self;
        }
        let r = roll((delta * 2) as usize) as i64 - delta;
        Timestamp(self.0 + r)
    }

    pub fn is_valid(&self, tolerance_seconds: i64) -> bool {
        let cur = Self::now().0;
        (cur - self.0).abs() <= tolerance_seconds
    }
}
