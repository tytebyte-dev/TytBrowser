// Module: common\antireplay\antireplay_test.rs
// 1:1 Rust unit test suite corresponding to Go common\antireplay\antireplay_test.go

#[cfg(test)]
mod tests {
    use super::super::mapfilter::ReplayFilter;

    #[test]
    fn test_map_filter() {
        let filter = ReplayFilter::<[u8; 16]>::new(120);
        let mut sample = [0u8; 16];
        rand::Rng::fill(&mut rand::thread_rng(), &mut sample);

        assert!(filter.check(sample));
        assert!(!filter.check(sample), "Unexpected true negative");

        sample[0] = sample[0].wrapping_add(1);
        assert!(filter.check(sample), "Unexpected false positive");
    }

    #[test]
    fn test_map_filter_rotation() {
        let filter = ReplayFilter::<String>::new(0); // 0s interval forces rotation on next check
        assert!(filter.check("item1".to_string()));
        // Still in pool_b after rotation
        assert!(!filter.check("item1".to_string()));
    }
}
