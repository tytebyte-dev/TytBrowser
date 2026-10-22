#[cfg(test)]
mod tests {
    use crate::app::observatory::Observatory;
    use std::time::Duration;

    #[test]
    fn test_observatory_latency_recording_and_best_selection() {
        let observatory = Observatory::new(
            "https://cp.cloudflare.com/generate_204",
            Duration::from_secs(60),
            vec!["node-*".into()],
        );

        observatory.record_result("node-hk", Duration::from_millis(150), true, None);
        observatory.record_result("node-jp", Duration::from_millis(80), true, None);
        observatory.record_result("node-us", Duration::from_millis(220), true, None);
        observatory.record_result(
            "node-dead",
            Duration::from_millis(0),
            false,
            Some("timeout".into()),
        );

        let hk = observatory.get_status("node-hk").unwrap();
        assert!(hk.alive);
        assert_eq!(hk.delay_ms, 150);

        let dead = observatory.get_status("node-dead").unwrap();
        assert!(!dead.alive);
        assert_eq!(dead.last_error_reason, Some("timeout".into()));

        let candidates = vec![
            "node-hk".to_string(),
            "node-jp".to_string(),
            "node-us".to_string(),
            "node-dead".to_string(),
        ];

        let best = observatory.get_best_outbound(&candidates);
        assert_eq!(best, Some("node-jp".to_string()));
    }
}
