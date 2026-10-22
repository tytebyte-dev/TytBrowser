// Module: features\stats\stats.rs
// 1:1 Rust implementation corresponding to Go features\stats\stats.go

use crate::features::feature::{Feature, TYPE_STATS_MANAGER};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

#[derive(Debug, Default)]
pub struct Counter {
    val: AtomicI64,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            val: AtomicI64::new(0),
        }
    }

    pub fn value(&self) -> i64 {
        self.val.load(Ordering::Relaxed)
    }

    pub fn set(&self, val: i64) -> i64 {
        self.val.swap(val, Ordering::Relaxed)
    }

    pub fn add(&self, delta: i64) -> i64 {
        self.val.fetch_add(delta, Ordering::Relaxed) + delta
    }
}

pub trait OnlineMap: Send + Sync {
    fn count(&self) -> usize;
    fn add_ip(&self, ip: &str);
    fn remove_ip(&self, ip: &str);
    fn list(&self) -> Vec<String>;
    fn ip_time_map(&self) -> HashMap<String, SystemTime>;
}

#[derive(Default)]
pub struct DefaultOnlineMap {
    entries: RwLock<HashMap<String, (usize, SystemTime)>>,
}

impl DefaultOnlineMap {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }
}

impl OnlineMap for DefaultOnlineMap {
    fn count(&self) -> usize {
        self.entries.read().map(|m| m.len()).unwrap_or(0)
    }

    fn add_ip(&self, ip: &str) {
        if let Ok(mut map) = self.entries.write() {
            let entry = map.entry(ip.to_string()).or_insert((0, SystemTime::now()));
            entry.0 += 1;
            entry.1 = SystemTime::now();
        }
    }

    fn remove_ip(&self, ip: &str) {
        if let Ok(mut map) = self.entries.write() {
            let mut remove = false;
            if let Some(entry) = map.get_mut(ip) {
                if entry.0 > 1 {
                    entry.0 -= 1;
                } else {
                    remove = true;
                }
            }
            if remove {
                map.remove(ip);
            }
        }
    }

    fn list(&self) -> Vec<String> {
        self.entries
            .read()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }

    fn ip_time_map(&self) -> HashMap<String, SystemTime> {
        self.entries
            .read()
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.1)).collect())
            .unwrap_or_default()
    }
}

pub trait Channel: Send + Sync {
    fn publish(&self, val: i64);
    fn subscriber_count(&self) -> usize;
}

#[derive(Clone)]
pub struct SimpleChannel {
    tx: tokio::sync::broadcast::Sender<i64>,
}

impl SimpleChannel {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(capacity.max(16));
        Self { tx }
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<i64> {
        self.tx.subscribe()
    }
}

impl Channel for SimpleChannel {
    fn publish(&self, val: i64) {
        let _ = self.tx.send(val);
    }

    fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

pub trait StatsManagerTrait: Feature {
    fn register_counter(&self, name: &str) -> Arc<Counter>;
    fn get_counter(&self, name: &str) -> Option<Arc<Counter>>;
    fn register_online_map(&self, name: &str) -> Arc<dyn OnlineMap>;
    fn get_online_map(&self, name: &str) -> Option<Arc<dyn OnlineMap>>;
    fn register_channel(&self, name: &str) -> Arc<dyn Channel> {
        let _ = name;
        Arc::new(SimpleChannel::new(32))
    }
    fn get_channel(&self, name: &str) -> Option<Arc<dyn Channel>> {
        let _ = name;
        None
    }
    fn get_all_online_users(&self) -> Vec<String>;
}

pub struct DefaultStatsManager {
    counters: RwLock<HashMap<String, Arc<Counter>>>,
    online_maps: RwLock<HashMap<String, Arc<dyn OnlineMap>>>,
    channels: RwLock<HashMap<String, Arc<dyn Channel>>>,
}

impl DefaultStatsManager {
    pub fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
            online_maps: RwLock::new(HashMap::new()),
            channels: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_counter(&self, name: &str) -> Arc<Counter> {
        StatsManagerTrait::register_counter(self, name)
    }

    pub fn get_counter(&self, name: &str) -> Option<Arc<Counter>> {
        StatsManagerTrait::get_counter(self, name)
    }

    pub fn register_online_map(&self, name: &str) -> Arc<dyn OnlineMap> {
        StatsManagerTrait::register_online_map(self, name)
    }

    pub fn get_online_map(&self, name: &str) -> Option<Arc<dyn OnlineMap>> {
        StatsManagerTrait::get_online_map(self, name)
    }

    pub fn register_channel(&self, name: &str) -> Arc<dyn Channel> {
        StatsManagerTrait::register_channel(self, name)
    }

    pub fn get_channel(&self, name: &str) -> Option<Arc<dyn Channel>> {
        StatsManagerTrait::get_channel(self, name)
    }

    pub fn get_all_online_users(&self) -> Vec<String> {
        StatsManagerTrait::get_all_online_users(self)
    }
}

impl Default for DefaultStatsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Feature for DefaultStatsManager {
    fn feature_type(&self) -> &'static str {
        TYPE_STATS_MANAGER
    }
}

impl StatsManagerTrait for DefaultStatsManager {
    fn register_counter(&self, name: &str) -> Arc<Counter> {
        let mut map = self.counters.write().unwrap();
        map.entry(name.to_string())
            .or_insert_with(|| Arc::new(Counter::new()))
            .clone()
    }

    fn get_counter(&self, name: &str) -> Option<Arc<Counter>> {
        self.counters.read().ok()?.get(name).cloned()
    }

    fn register_online_map(&self, name: &str) -> Arc<dyn OnlineMap> {
        let mut map = self.online_maps.write().unwrap();
        map.entry(name.to_string())
            .or_insert_with(|| Arc::new(DefaultOnlineMap::new()))
            .clone()
    }

    fn get_online_map(&self, name: &str) -> Option<Arc<dyn OnlineMap>> {
        self.online_maps.read().ok()?.get(name).cloned()
    }

    fn register_channel(&self, name: &str) -> Arc<dyn Channel> {
        let mut map = self.channels.write().unwrap();
        map.entry(name.to_string())
            .or_insert_with(|| Arc::new(SimpleChannel::new(32)))
            .clone()
    }

    fn get_channel(&self, name: &str) -> Option<Arc<dyn Channel>> {
        self.channels.read().ok()?.get(name).cloned()
    }

    fn get_all_online_users(&self) -> Vec<String> {
        let map = self.online_maps.read().unwrap();
        map.keys().cloned().collect()
    }
}

// Keep StatsManager struct directly exposing register_counter for scenarios and backward compatibility
pub type StatsManager = DefaultStatsManager;

#[derive(Default, Clone, Debug)]
pub struct NoopStatsManager;

impl NoopStatsManager {
    pub fn new() -> Self {
        Self
    }

    pub fn register_counter(&self, name: &str) -> Arc<Counter> {
        StatsManagerTrait::register_counter(self, name)
    }

    pub fn get_counter(&self, name: &str) -> Option<Arc<Counter>> {
        StatsManagerTrait::get_counter(self, name)
    }

    pub fn register_online_map(&self, name: &str) -> Arc<dyn OnlineMap> {
        StatsManagerTrait::register_online_map(self, name)
    }

    pub fn get_online_map(&self, name: &str) -> Option<Arc<dyn OnlineMap>> {
        StatsManagerTrait::get_online_map(self, name)
    }

    pub fn get_all_online_users(&self) -> Vec<String> {
        StatsManagerTrait::get_all_online_users(self)
    }
}

impl Feature for NoopStatsManager {
    fn feature_type(&self) -> &'static str {
        TYPE_STATS_MANAGER
    }
}

impl StatsManagerTrait for NoopStatsManager {
    fn register_counter(&self, _name: &str) -> Arc<Counter> {
        Arc::new(Counter::new())
    }

    fn get_counter(&self, _name: &str) -> Option<Arc<Counter>> {
        None
    }

    fn register_online_map(&self, _name: &str) -> Arc<dyn OnlineMap> {
        Arc::new(DefaultOnlineMap::new())
    }

    fn get_online_map(&self, _name: &str) -> Option<Arc<dyn OnlineMap>> {
        None
    }

    fn get_all_online_users(&self) -> Vec<String> {
        Vec::new()
    }
}

pub fn get_or_register_counter(mgr: &dyn StatsManagerTrait, name: &str) -> Arc<Counter> {
    if let Some(c) = mgr.get_counter(name) {
        c
    } else {
        mgr.register_counter(name)
    }
}

pub fn get_or_register_online_map(mgr: &dyn StatsManagerTrait, name: &str) -> Arc<dyn OnlineMap> {
    if let Some(m) = mgr.get_online_map(name) {
        m
    } else {
        mgr.register_online_map(name)
    }
}

pub fn get_or_register_channel(mgr: &dyn StatsManagerTrait, name: &str) -> Arc<dyn Channel> {
    if let Some(c) = mgr.get_channel(name) {
        c
    } else {
        mgr.register_channel(name)
    }
}

pub fn manager_type() -> &'static str {
    TYPE_STATS_MANAGER
}
