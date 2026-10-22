// Module: app\observatory\burst\healthping_result.rs
// 1:1 Rust implementation corresponding to Go app\observatory\burst\healthping_result.go

use super::burst::RTT_FAILED;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HealthPingStats {
    pub all: usize,
    pub fail: usize,
    pub deviation: Duration,
    pub average: Duration,
    pub max: Duration,
    pub min: Duration,
}

#[derive(Debug, Clone)]
struct PingRTT {
    time: Option<Instant>,
    value: Duration,
}

impl Default for PingRTT {
    fn default() -> Self {
        Self {
            time: None,
            value: Duration::ZERO,
        }
    }
}

pub struct HealthPingRTTS {
    idx: isize,
    capacity: usize,
    validity: Duration,
    rtts: Vec<PingRTT>,
    last_update_at: Option<Instant>,
    stats: Option<HealthPingStats>,
}

impl HealthPingRTTS {
    pub fn new(capacity: usize, validity: Duration) -> Self {
        Self {
            idx: -1,
            capacity,
            validity,
            rtts: Vec::new(),
            last_update_at: None,
            stats: None,
        }
    }

    pub fn put(&mut self, d: Duration) {
        if self.rtts.is_empty() {
            self.rtts = vec![PingRTT::default(); self.capacity];
            self.idx = -1;
        }
        self.idx = self.calc_index(1);
        let now = Instant::now();
        let idx = self.idx as usize;
        self.rtts[idx].time = Some(now);
        self.rtts[idx].value = d;
    }

    fn calc_index(&self, step: usize) -> isize {
        if self.idx < 0 {
            0
        } else {
            ((self.idx as usize + step) % self.capacity) as isize
        }
    }

    pub fn get(&mut self) -> HealthPingStats {
        self.get_statistics()
    }

    pub fn get_statistics(&self) -> HealthPingStats {
        let mut stats = HealthPingStats {
            all: 0,
            fail: 0,
            deviation: Duration::ZERO,
            average: Duration::ZERO,
            max: Duration::ZERO,
            min: RTT_FAILED,
        };

        let now = Instant::now();
        let mut sum_nanos = 0u64;
        let mut cnt = 0usize;
        let mut valid_rtts = Vec::new();

        for rtt in &self.rtts {
            match rtt.time {
                Some(t)
                    if now.duration_since(t) <= self.validity && rtt.value != Duration::ZERO =>
                {
                    if rtt.value == RTT_FAILED {
                        stats.fail += 1;
                    } else {
                        cnt += 1;
                        sum_nanos += rtt.value.as_nanos() as u64;
                        valid_rtts.push(rtt.value);
                        if stats.max < rtt.value {
                            stats.max = rtt.value;
                        }
                        if stats.min > rtt.value {
                            stats.min = rtt.value;
                        }
                    }
                }
                _ => continue,
            }
        }

        stats.all = cnt + stats.fail;
        if cnt == 0 {
            stats.min = Duration::ZERO;
            return stats;
        }

        let avg_nanos = sum_nanos / (cnt as u64);
        stats.average = Duration::from_nanos(avg_nanos);

        let std_nanos = if cnt < 2 {
            avg_nanos / 2
        } else {
            let mut variance = 0.0;
            let avg_f = avg_nanos as f64;
            for rtt in valid_rtts {
                let diff = (rtt.as_nanos() as f64) - avg_f;
                variance += diff * diff;
            }
            (variance / (cnt as f64)).sqrt() as u64
        };

        stats.deviation = Duration::from_nanos(std_nanos);
        stats
    }

    pub fn get_with_cache(&mut self) -> HealthPingStats {
        let now = Instant::now();
        let should_update = match (self.stats.as_ref(), self.last_update_at, self.idx) {
            (None, _, _) => true,
            (_, Some(last_update), idx) if idx >= 0 => {
                if let Some(put_time) = self.rtts[idx as usize].time {
                    last_update < put_time || self.find_outdated(now)
                } else {
                    true
                }
            }
            _ => true,
        };

        if should_update {
            let s = self.get_statistics();
            self.stats = Some(s.clone());
            self.last_update_at = Some(now);
            s
        } else {
            self.stats.clone().unwrap()
        }
    }

    fn find_outdated(&self, now: Instant) -> bool {
        for i in (self.capacity - 1)..(2 * self.capacity) {
            let idx = i % self.capacity;
            if let Some(t) = self.rtts[idx].time
                && now.duration_since(t) > self.validity
            {
                return true;
            }
        }
        false
    }
}

pub fn new_health_ping_result(capacity: usize, validity: Duration) -> HealthPingRTTS {
    HealthPingRTTS::new(capacity, validity)
}

// Backward compatible alias
pub type HealthPingResult = HealthPingRTTS;
