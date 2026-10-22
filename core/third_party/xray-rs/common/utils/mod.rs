pub mod access_field;
pub mod browser;
pub mod padding;
pub mod typed_sync_map;

use rand::Rng;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::RwLock;

const BASE62_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const H2_PACK_CORRECTION_FACTOR: f64 = 1.2493702770780857;

pub fn h2_base62_pad(expected_len: usize) -> String {
    let actual_len = (expected_len as f64 * H2_PACK_CORRECTION_FACTOR) as usize;
    let mut rng = rand::thread_rng();
    let mut result = Vec::with_capacity(actual_len);
    for _ in 0..actual_len {
        let idx = rng.gen_range(0..BASE62_CHARS.len());
        result.push(BASE62_CHARS[idx]);
    }
    String::from_utf8(result).unwrap_or_default()
}

pub struct TypedSyncMap<K, V> {
    inner: RwLock<HashMap<K, V>>,
}

impl<K, V> TypedSyncMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub fn load(&self, key: &K) -> Option<V> {
        self.inner.read().ok()?.get(key).cloned()
    }

    pub fn store(&self, key: K, val: V) {
        if let Ok(mut guard) = self.inner.write() {
            guard.insert(key, val);
        }
    }

    pub fn delete(&self, key: &K) -> Option<V> {
        let mut guard = self.inner.write().ok()?;
        guard.remove(key)
    }

    pub fn len(&self) -> usize {
        self.inner.read().map(|g| g.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K, V> Default for TypedSyncMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_h2_base62_padding_generation() {
        let pad = h2_base62_pad(10);
        assert!(!pad.is_empty());
        assert!(pad.len() >= 10);
    }

    #[test]
    fn test_typed_sync_map_operations() {
        let map = TypedSyncMap::<String, i32>::new();
        assert!(map.is_empty());
        map.store("key1".into(), 42);
        assert_eq!(map.len(), 1);
        assert_eq!(map.load(&"key1".into()), Some(42));
        assert_eq!(map.delete(&"key1".into()), Some(42));
        assert!(map.is_empty());
    }
}
