// Module: app\log\log_test.rs
// 1:1 Rust unit test suite corresponding to Go app\log\log_test.go

#[cfg(test)]
mod tests {
    use super::super::config_pb::{Config as LogConfigPb, LogType as PbLogType};
    use super::super::{LogCreatorOptions, LogLevel, LogManager, create_logger};
    use crate::common::log::Severity;

    #[test]
    fn test_log_manager_creation() {
        let lm = LogManager::new(LogLevel::Info, 0, 0);
        assert_eq!(lm.get_level(), LogLevel::Info);
        lm.set_level(LogLevel::Debug);
        assert_eq!(lm.get_level(), LogLevel::Debug);
    }

    #[test]
    fn test_log_config_pb_and_creator() {
        let mut cfg = LogConfigPb::default();
        cfg.error_log_type = PbLogType::Console;
        cfg.error_log_level = Severity::Debug;
        cfg.enable_dns_log = true;
        cfg.mask_address = "half".to_string();

        assert_eq!(cfg.error_log_type, PbLogType::Console);
        assert_eq!(cfg.error_log_level, Severity::Debug);
        assert!(cfg.enable_dns_log);

        let opts = LogCreatorOptions::console(LogLevel::Debug);
        let logger = create_logger(&opts);
        assert!(logger.is_ok());

        let file_opts = LogCreatorOptions::file("test_log.log", LogLevel::Info);
        assert!(create_logger(&file_opts).is_ok());

        let none_opts = LogCreatorOptions::none();
        assert!(create_logger(&none_opts).is_ok());
    }
}
