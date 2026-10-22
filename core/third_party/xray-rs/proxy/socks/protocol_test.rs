// Module: proxy\socks\protocol_test.rs
// 1:1 Rust unit test suite corresponding to Go proxy\socks\protocol_test.go

#[cfg(test)]
mod tests {
    use super::super::protocol::SocksProtocol;
    use super::super::socks::{VERSION_5, PROTOCOL_NAME};

    #[test]
    fn test_socks_constants() {
        assert_eq!(VERSION_5, 5);
        assert_eq!(PROTOCOL_NAME, "socks");
    }

    #[test]
    fn test_socks_protocol_addr_type() {
        assert_eq!(SocksProtocol::addr_type_v4(), 0x01);
        assert_eq!(SocksProtocol::addr_type_domain(), 0x03);
        assert_eq!(SocksProtocol::addr_type_v6(), 0x04);
    }
}
