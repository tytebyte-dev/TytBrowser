#[cfg(test)]
mod tests {
    use crate::common::cache::LruCache;
    use crate::common::ocsp::OcspCache;
    use crate::common::peer::AverageLatency;
    use std::time::Duration;

    #[test]
    fn test_lru_cache_capacity_and_eviction() {
        let cache = LruCache::<String, i32>::new(2);
        cache.put("a".into(), 1);
        cache.put("b".into(), 2);

        assert_eq!(cache.peek_key_from_value(&2), Some("b".into()));
        assert_eq!(cache.get(&"a".into()), Some(1));

        // Add 3rd item, should evict 'b' because 'a' was recently accessed
        cache.put("c".into(), 3);
        assert_eq!(cache.get(&"b".into()), None);
        assert_eq!(cache.get(&"a".into()), Some(1));
        assert_eq!(cache.get(&"c".into()), Some(3));
    }

    #[test]
    fn test_average_latency_exponential_moving_average() {
        let avg = AverageLatency::new(100);
        assert_eq!(avg.value(), 100);

        // (100 + 40 * 2) / 3 = 180 / 3 = 60
        avg.update(40);
        assert_eq!(avg.value(), 60);
    }

    #[test]
    fn test_ocsp_cache_expiry() {
        let ocsp = OcspCache::new();
        assert_eq!(ocsp.get(), None);

        let data = vec![1, 2, 3, 4];
        ocsp.set(data.clone(), Duration::from_secs(3600));
        assert_eq!(ocsp.get(), Some(data));
    }
}
