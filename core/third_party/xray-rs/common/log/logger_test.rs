// Module: common\log\logger_test.rs
// 1:1 Rust unit test suite corresponding to Go common\log\logger_test.go

#[cfg(test)]
mod tests {
    use crate::common::log::logger::{Logger, create_file_log_writer, new_logger};
    use crate::common::log::{GeneralMessage, Handler, LogLevel, Severity};
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[test]
    fn test_logger_level_filtering() {
        let logger = Logger::new(LogLevel::Warning);
        assert!(logger.is_enabled(LogLevel::Error));
        assert!(logger.is_enabled(LogLevel::Warning));
        assert!(!logger.is_enabled(LogLevel::Info));
        assert!(!logger.is_enabled(LogLevel::Debug));
    }

    #[test]
    fn test_file_logger() {
        let temp_dir = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = temp_dir.join(format!("xray_test_log_{}.log", nanos));
        let path_str = path.to_str().unwrap();

        let creator = create_file_log_writer(path_str).expect("create file log writer");
        let handler = new_logger(creator);

        handler.handle(&GeneralMessage {
            severity: Severity::Info,
            content: "Test Log".to_string(),
        });

        // Give the background worker a moment to process and flush
        thread::sleep(Duration::from_millis(200));
        handler.close();

        let content = std::fs::read_to_string(&path).expect("read log file");
        let _ = std::fs::remove_file(&path);

        assert!(
            content.contains("Test Log"),
            "Expect log text contains 'Test Log', but actually: {}",
            content
        );
    }
}
