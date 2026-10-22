// Module: app\\observatory\\observatory_test.rs
// 1:1 Rust unit test suite corresponding to Go app\\observatory

#[cfg(test)]
mod tests {
    use super::super::Observatory;
    use std::time::Duration;

    #[test]
    fn test_observatory_records() {
        let obs = Observatory::new(
            "https://www.google.com/gen_204",
            Duration::from_secs(10),
            vec![],
        );
        assert_eq!(obs.probe_url(), "https://www.google.com/gen_204");
        obs.record_status("tag-1", true, 50, None);
        let statuses = obs.all_statuses();
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].outbound_tag, "tag-1");
        assert!(statuses[0].alive);
    }
}
