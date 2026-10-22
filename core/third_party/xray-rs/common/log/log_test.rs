// Module: common\log\log_test.rs
// 1:1 Rust unit test suite corresponding to Go common\log\log_test.go

#[cfg(test)]
mod tests {
    use crate::common::ctx::Context;
    use crate::common::log::{
        self, AccessMessage, AccessStatus, DnsLog, DnsStatus, GeneralMessage, Handler, Message,
        Severity, access_message_from_context, context_with_access_message,
    };
    use crate::common::net;
    use std::net::IpAddr;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    struct TestLogger {
        value: Mutex<String>,
    }

    impl Handler for TestLogger {
        fn handle(&self, msg: &dyn Message) {
            *self.value.lock().unwrap() = msg.to_log_string();
        }
    }

    #[test]
    fn test_log_record() {
        let logger = Arc::new(TestLogger {
            value: Mutex::new(String::new()),
        });
        log::register_handler(logger.clone());

        let ip = "8.8.8.8";
        let parsed = net::parse_address(ip);
        log::record(&GeneralMessage {
            severity: Severity::Error,
            content: parsed.to_string(),
        });

        let val = logger.value.lock().unwrap().clone();
        assert_eq!(val, format!("[Error] {}", ip));
    }

    #[test]
    fn test_access_message_and_context() {
        let mut msg =
            AccessMessage::new("127.0.0.1:12345", "example.com:443", AccessStatus::Accepted);
        msg.detour = "proxy".to_string();
        msg.reason = "matched-domain".to_string();
        msg.email = "admin@example.com".to_string();

        let str_rep = msg.to_log_string();
        assert_eq!(
            str_rep,
            "from 127.0.0.1:12345 accepted example.com:443 [proxy] matched-domain email: admin@example.com"
        );

        let ctx = Context::new();
        context_with_access_message(&ctx, Arc::new(msg.clone()));
        let retrieved = access_message_from_context(&ctx).expect("retrieve access message");
        assert_eq!(retrieved.from, "127.0.0.1:12345");
        assert_eq!(retrieved.to, "example.com:443");
        assert_eq!(retrieved.status, AccessStatus::Accepted);
    }

    #[test]
    fn test_dns_log_formatting() {
        let mut dns_log = DnsLog::new("8.8.8.8", "example.com");
        dns_log.status = DnsStatus::Queried;
        dns_log.result = vec!["93.184.216.34".parse::<IpAddr>().unwrap()];
        dns_log.elapsed = Duration::from_millis(25);
        dns_log.error = Some("warning-timeout".to_string());

        let log_str = dns_log.to_log_string();
        assert!(log_str.starts_with("8.8.8.8 got answer: example.com -> [93.184.216.34]"));
        assert!(log_str.contains("<warning-timeout>"));
    }
}
