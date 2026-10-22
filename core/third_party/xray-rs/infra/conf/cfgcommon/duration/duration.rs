// Module: infra\\conf\\cfgcommon\\duration\\duration.rs
// 1:1 Rust implementation corresponding to Go infra\\conf\\cfgcommon\\duration\\duration.go

use std::time::Duration;

pub fn parse_duration(s: &str) -> Option<Duration> {
    let s = s.trim();
    if let Some(ms) = s.strip_suffix("ms") {
        ms.trim().parse::<u64>().ok().map(Duration::from_millis)
    } else if let Some(sec) = s.strip_suffix('s') {
        sec.trim().parse::<u64>().ok().map(Duration::from_secs)
    } else if let Some(min) = s.strip_suffix('m') {
        min.trim()
            .parse::<u64>()
            .ok()
            .map(|m| Duration::from_secs(m * 60))
    } else if let Some(hr) = s.strip_suffix('h') {
        hr.trim()
            .parse::<u64>()
            .ok()
            .map(|h| Duration::from_secs(h * 3600))
    } else {
        s.parse::<u64>().ok().map(Duration::from_secs)
    }
}
