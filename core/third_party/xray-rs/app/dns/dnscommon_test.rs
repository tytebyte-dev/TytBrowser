// Module: app\\dns\\dnscommon_test.rs
// 1:1 Rust unit test suite corresponding to Go app\\dns\\dnscommon_test.go

#[cfg(test)]
mod tests {
    use super::super::dnscommon::{IPRecord, fqdn};
    use std::time::Duration;

    #[test]
    fn test_fqdn_and_ip_record() {
        assert_eq!(fqdn("example.com"), "example.com.");
        assert_eq!(fqdn("example.com."), "example.com.");

        let rec = IPRecord::new(
            1,
            vec!["1.1.1.1".parse().unwrap()],
            Duration::from_secs(60),
            0,
        );
        assert!(!rec.is_expired());
        assert_eq!(rec.get_ips().unwrap().len(), 1);
    }
}
