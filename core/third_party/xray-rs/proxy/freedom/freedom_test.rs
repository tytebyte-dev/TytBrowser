// Module: proxy\freedom\freedom_test.rs
#[cfg(test)]
mod tests {
    use super::super::config::FreedomConfig;

    #[test]
    fn test_freedom_config() {
        let cfg = FreedomConfig {
            domain_strategy: "AsIs".into(),
            timeout: 30,
        };
        assert_eq!(cfg.domain_strategy, "AsIs");
    }
}
