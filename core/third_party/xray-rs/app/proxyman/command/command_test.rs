// Module: app\\proxyman\\command\\command_test.rs
// 1:1 Rust unit test suite corresponding to Go app\\proxyman\\command\\command_test.go

#[cfg(test)]
mod tests {
    use super::super::command::ProxymanCommandService;

    #[test]
    fn test_proxyman_command_handler_service() {
        let svc = ProxymanCommandService::new();
        assert!(svc.add_inbound("in-1"));
        assert!(svc.has_inbound("in-1"));
        assert!(svc.remove_inbound("in-1"));
        assert!(!svc.has_inbound("in-1"));

        assert!(svc.add_outbound("out-1"));
        assert!(svc.has_outbound("out-1"));
        assert!(svc.remove_outbound("out-1"));
        assert!(!svc.has_outbound("out-1"));
    }
}
