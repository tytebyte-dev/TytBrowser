// Module: infra\conf\loopback_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\loopback_test.go

#[cfg(test)]
mod tests {
    use super::super::loopback::LoopbackConfig;

    #[test]
    fn test_loopback_config() {
        let cfg = LoopbackConfig::new("tag-loop");
        assert_eq!(cfg.inbound_tag, "tag-loop");
    }
}
