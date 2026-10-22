// Module: common\cache\lru_test.rs
// 1:1 Rust unit test suite corresponding to Go common\cache\lru_test.go

#[cfg(test)]
mod tests {
    use super::super::lru::{Lru, LruCache, new_lru};

    #[test]
    fn test_lru_trait() {
        let lru: Box<dyn Lru<i32, i32>> = Box::new(LruCache::new(2));
        lru.put(1, 10);
        assert_eq!(lru.get(&1), Some(10));
    }

    #[test]
    fn test_lru_replace_value() {
        let lru = new_lru(2);
        lru.put(2, 6);
        lru.put(1, 5);
        lru.put(1, 2);

        assert_eq!(lru.get(&1), Some(2));
        assert_eq!(lru.get(&2), Some(6));
    }

    #[test]
    fn test_lru_remove_old() {
        let lru = new_lru(2);
        assert_eq!(lru.get(&2), None);

        lru.put(1, 1);
        lru.put(2, 2);
        assert_eq!(lru.get(&1), Some(1));

        lru.put(3, 3);
        assert_eq!(lru.get(&2), None);

        lru.put(4, 4);
        assert_eq!(lru.get(&1), None);
        assert_eq!(lru.get(&3), Some(3));
        assert_eq!(lru.get(&4), Some(4));
    }

    #[test]
    fn test_get_key_from_value() {
        let lru = new_lru(2);
        lru.put(3, 3);
        lru.put(2, 2);
        lru.get_key_from_value(&3); // brings 3 to front

        lru.put(1, 1); // should evict 2
        assert_eq!(lru.get_key_from_value(&2), None);
        assert_eq!(lru.get_key_from_value(&3), Some(3));
    }

    #[test]
    fn test_peek_key_from_value() {
        let lru = new_lru(2);
        lru.put(3, 3);
        lru.put(2, 2);
        lru.peek_key_from_value(&3); // does NOT bring 3 to front

        lru.put(1, 1); // should evict 3
        assert_eq!(lru.peek_key_from_value(&3), None);
        assert_eq!(lru.peek_key_from_value(&2), Some(2));
    }
}
