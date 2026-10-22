// Module: app\\dns\\nameserver_quic_test.rs
// 1:1 Rust unit test suite corresponding to Go app\\dns\\nameserver_quic_test.go

#[cfg(test)]
mod tests {
    use super::super::nameserver_quic::QuicNameServer;

    #[test]
    fn test_quic_nameserver_creation() {
        let quic = QuicNameServer::new("dns.adguard.com:853");
        assert_eq!(quic.url(), "dns.adguard.com:853");
    }
}
