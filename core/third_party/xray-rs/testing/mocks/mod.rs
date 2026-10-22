// Module: testing\mocks\mod.rs

pub mod dns;
pub mod io;
pub mod log;
pub mod mux;
pub mod outbound;
pub mod proxy;

pub use dns::MockDnsServer;
pub use io::MockIoPair;
pub use log::MockLogCollector;
pub use mux::MockMuxSession;
pub use outbound::MockOutboundHandler;
pub use proxy::{MockInboundHandler, MockProxyHandler};

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_mock_dns() {
        let dns = MockDnsServer::new();
        let ip = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
        dns.set_record("example.com", vec![ip]);
        assert_eq!(dns.resolve("example.com"), Some(vec![ip]));
        assert_eq!(dns.resolve("unknown.com"), None);
    }

    #[test]
    fn test_mock_log_collector() {
        let collector = MockLogCollector::new();
        collector.log("hello");
        collector.log("world");
        assert_eq!(collector.total_logs(), 2);
    }

    #[test]
    fn test_mock_mux_session() {
        let mux = MockMuxSession::new();
        assert_eq!(mux.allocate_id(), 1);
        assert_eq!(mux.allocate_id(), 2);
    }

    #[test]
    fn test_mock_handlers() {
        let ob = MockOutboundHandler::new("mock-out");
        assert_eq!(ob.tag(), "mock-out");
        ob.count_invocation();
        assert_eq!(ob.invocations(), 1);

        let ib = MockInboundHandler::new("mock-in");
        assert_eq!(ib.tag(), "mock-in");
    }
}
