// Module: proxy\dns\dns_test.rs
#[cfg(test)]
mod tests {
    use super::super::DefaultDnsHandler;

    #[test]
    fn test_dns_proxy_creation() {
        let h = DefaultDnsHandler::new("dns-out");
        assert_eq!(h.tag, "dns-out");
    }
}
