// Module: common\cache\lru.rs
// 1:1 Rust implementation corresponding to Go common\cache\lru.go

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::Mutex;

/// Lru trait defines bidirectional key/value lookup and access-order tracking.
pub trait Lru<K, V> {
    fn get(&self, key: &K) -> Option<V>;
    fn get_key_from_value(&self, val: &V) -> Option<K>;
    fn peek_key_from_value(&self, val: &V) -> Option<K>;
    fn put(&self, key: K, val: V);
}

/// LruCache is a thread-safe bidirectional LRU cache.
pub struct LruCache<K, V> {
    capacity: usize,
    entries: Mutex<LruInner<K, V>>,
}

struct LruInner<K, V> {
    key_to_val: HashMap<K, V>,
    val_to_key: HashMap<V, K>,
    order: VecDeque<K>,
}

impl<K, V> LruCache<K, V>
where
    K: Clone + Eq + Hash + Send + 'static,
    V: Clone + Eq + Hash + Send + 'static,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            entries: Mutex::new(LruInner {
                key_to_val: HashMap::with_capacity(capacity),
                val_to_key: HashMap::with_capacity(capacity),
                order: VecDeque::with_capacity(capacity),
            }),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let mut inner = self.entries.lock().ok()?;
        if let Some(val) = inner.key_to_val.get(key).cloned() {
            if let Some(pos) = inner.order.iter().position(|x| x == key) {
                inner.order.remove(pos);
                inner.order.push_back(key.clone());
            }
            Some(val)
        } else {
            None
        }
    }

    pub fn get_key_from_value(&self, val: &V) -> Option<K> {
        let mut inner = self.entries.lock().ok()?;
        if let Some(key) = inner.val_to_key.get(val).cloned() {
            if let Some(pos) = inner.order.iter().position(|x| x == &key) {
                inner.order.remove(pos);
                inner.order.push_back(key.clone());
            }
            Some(key)
        } else {
            None
        }
    }

    pub fn peek_key_from_value(&self, val: &V) -> Option<K> {
        let inner = self.entries.lock().ok()?;
        inner.val_to_key.get(val).cloned()
    }

    pub fn put(&self, key: K, val: V) {
        if let Ok(mut inner) = self.entries.lock() {
            if inner.key_to_val.contains_key(&key) {
                if let Some(old_val) = inner.key_to_val.remove(&key) {
                    inner.val_to_key.remove(&old_val);
                }
                if let Some(pos) = inner.order.iter().position(|x| x == &key) {
                    inner.order.remove(pos);
                }
            } else if inner.order.len() >= self.capacity
                && let Some(lru_key) = inner.order.pop_front()
                && let Some(old_val) = inner.key_to_val.remove(&lru_key)
            {
                inner.val_to_key.remove(&old_val);
            }

            inner.key_to_val.insert(key.clone(), val.clone());
            inner.val_to_key.insert(val, key.clone());
            inner.order.push_back(key);
        }
    }
}

impl<K, V> Lru<K, V> for LruCache<K, V>
where
    K: Clone + Eq + Hash + Send + 'static,
    V: Clone + Eq + Hash + Send + 'static,
{
    fn get(&self, key: &K) -> Option<V> {
        self.get(key)
    }

    fn get_key_from_value(&self, val: &V) -> Option<K> {
        self.get_key_from_value(val)
    }

    fn peek_key_from_value(&self, val: &V) -> Option<K> {
        self.peek_key_from_value(val)
    }

    fn put(&self, key: K, val: V) {
        self.put(key, val)
    }
}

pub fn new_lru<K, V>(capacity: usize) -> LruCache<K, V>
where
    K: Clone + Eq + Hash + Send + 'static,
    V: Clone + Eq + Hash + Send + 'static,
{
    LruCache::new(capacity)
}
