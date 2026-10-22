// Module: proxy\loopback\loopback_test.rs
#[cfg(test)]
mod tests {
    use super::super::config::LoopbackConfig;

    #[test]
    fn test_loopback_config() {
        let cfg = LoopbackConfig {
            inbound_tag: "in-1".into(),
        };
        assert_eq!(cfg.inbound_tag, "in-1");
    }
}
