// Module: transport\internet\finalmask\noise\conn.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\noise\conn.go

use super::config::NoiseConfig;
use rand::Rng;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct NoiseGenerator {
    pub min_len: usize,
    pub max_len: usize,
    pub min_delay: Duration,
    pub max_delay: Duration,
}

impl NoiseGenerator {
    pub fn new(config: NoiseConfig) -> Self {
        Self {
            min_len: config.min_len,
            max_len: config.max_len,
            min_delay: config.min_delay,
            max_delay: config.max_delay,
        }
    }

    pub fn generate_noise_packet(&self) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let min_sz = std::cmp::max(1, self.min_len);
        let max_sz = std::cmp::max(min_sz, self.max_len);
        let len = rng.gen_range(min_sz..=max_sz);
        let mut buf = vec![0u8; len];
        rng.fill(&mut buf[..]);
        buf
    }
}

pub struct NoisePacketConn {
    pub config: NoiseConfig,
    sessions: Arc<Mutex<HashMap<String, Instant>>>,
}

impl NoisePacketConn {
    pub fn new(config: NoiseConfig) -> Self {
        Self {
            config,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Checks if noise packets should be injected for the given destination address.
    pub fn should_send_noise(&self, addr_str: &str) -> bool {
        let map = self.sessions.lock().unwrap();
        let now = Instant::now();
        if let Some(&expires) = map.get(addr_str)
            && now < expires
        {
            return false;
        }
        true
    }

    /// Generates the sequence of noise packets and their respective inter-packet delays.
    pub fn generate_noise_packets(&self) -> Vec<(Vec<u8>, Duration)> {
        let mut rng = rand::thread_rng();
        let mut packets = Vec::with_capacity(self.config.items.len());

        for item in &self.config.items {
            let pkt = if item.rand_max > 0 {
                let min_sz = std::cmp::max(0, item.rand_min) as usize;
                let max_sz = std::cmp::max(min_sz, item.rand_max as usize);
                let size = if max_sz > min_sz {
                    rng.gen_range(min_sz..=max_sz)
                } else {
                    min_sz
                };
                let r_min = item.rand_range_min.clamp(0, 255) as u8;
                let r_max = item.rand_range_max.clamp(r_min as i32, 255) as u8;
                let mut buf = vec![0u8; size];
                for b in &mut buf {
                    *b = if r_max > r_min {
                        rng.gen_range(r_min..=r_max)
                    } else {
                        r_min
                    };
                }
                buf
            } else {
                item.packet.clone()
            };

            let delay_ms = if item.delay_max > item.delay_min {
                rng.gen_range(item.delay_min..=item.delay_max)
            } else {
                item.delay_min
            };
            let delay = Duration::from_millis(std::cmp::max(0, delay_ms) as u64);

            packets.push((pkt, delay));
        }

        packets
    }

    /// Marks that noise packets were dispatched to `addr_str`, setting reset TTL.
    pub fn mark_sent(&self, addr_str: &str) {
        let mut map = self.sessions.lock().unwrap();
        let mut rng = rand::thread_rng();
        let reset_secs = if self.config.reset_max > self.config.reset_min {
            rng.gen_range(self.config.reset_min..=self.config.reset_max)
        } else {
            self.config.reset_min
        };
        let ttl = Duration::from_secs(std::cmp::max(1, reset_secs) as u64);
        map.insert(addr_str.to_string(), Instant::now() + ttl);
    }

    /// Purges expired address entries.
    pub fn clean_expired(&self) {
        let mut map = self.sessions.lock().unwrap();
        let now = Instant::now();
        map.retain(|_, &mut exp| exp > now);
    }
}
