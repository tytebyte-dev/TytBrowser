// Module: common\net\find_process_test.rs

#[cfg(test)]
mod tests {
    use crate::app::router::condition::Rule;
    use crate::common::net::{Address, Destination, ProcessFinder};
    use crate::common::protocol::SessionContext;
    use std::net::{Ipv4Addr, TcpListener};

    #[test]
    fn test_find_process_current_pid() {
        let cur_pid = std::process::id();
        #[cfg(target_os = "windows")]
        {
            let name = crate::common::net::find_process_windows::find_process_name_by_pid(cur_pid);
            assert!(
                name.is_some(),
                "Expected current process name to be resolved"
            );
        }
    }

    #[test]
    fn test_find_process_by_listening_tcp_socket() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind test TCP socket");
        let local_addr = listener.local_addr().expect("Failed to get local addr");

        #[cfg(target_os = "windows")]
        {
            let proc_info = ProcessFinder::find_process_by_socket(local_addr, true)
                .expect("Search should succeed");
            assert!(
                proc_info.is_some(),
                "Expected process info for local TCP socket"
            );
            let info = proc_info.unwrap();
            assert_eq!(
                info.pid,
                std::process::id(),
                "Owning PID should match current process"
            );
        }
    }

    #[test]
    fn test_rule_matches_process_self() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind test socket");
        let local_addr = listener.local_addr().expect("Failed to get local addr");

        let mut rule = Rule::new("direct");
        rule.process = vec!["self/".into(), "xray.exe".into(), "v2ray.exe".into()];

        let dest = Destination::new(Address::Ipv4(Ipv4Addr::new(1, 1, 1, 1)), 443);
        let mut session = SessionContext::new("tun-in", dest);
        session.source = Some(local_addr);

        #[cfg(target_os = "windows")]
        {
            assert!(
                rule.matches(&session),
                "Rule with self/ must match current process socket"
            );
        }
    }

    #[test]
    fn test_rule_rejects_unmatched_process() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind test socket");
        let local_addr = listener.local_addr().expect("Failed to get local addr");

        let mut rule = Rule::new("direct");
        rule.process = vec!["some_nonexistent_process_12345.exe".into()];

        let dest = Destination::new(Address::Ipv4(Ipv4Addr::new(1, 1, 1, 1)), 443);
        let mut session = SessionContext::new("tun-in", dest);
        session.source = Some(local_addr);

        assert!(
            !rule.matches(&session),
            "Rule with non-matching process must not match"
        );
    }
}
