#[cfg(test)]
mod tests {
    use crate::app::dns::FakeDnsHolder;
    use std::net::IpAddr;

    #[test]
    fn test_fakedns_pool_allocation_and_reverse_lookup() {
        let holder = FakeDnsHolder::new("198.18.0.0/15").unwrap();

        let domain1 = "google.com";
        let domain2 = "youtube.com";

        let fake_ip1 = holder.get_fake_ip_for_domain(domain1);
        let fake_ip2 = holder.get_fake_ip_for_domain(domain2);

        assert_ne!(fake_ip1, fake_ip2);
        assert!(holder.is_fake_ip(&IpAddr::V4(fake_ip1)));
        assert!(holder.is_fake_ip(&IpAddr::V4(fake_ip2)));

        // Repeated query returns the same allocated IP
        assert_eq!(holder.get_fake_ip_for_domain(domain1), fake_ip1);

        // Reverse lookup resolves back to domain
        assert_eq!(
            holder.get_domain_for_fake_ip(&fake_ip1),
            Some(domain1.to_string())
        );
        assert_eq!(
            holder.get_domain_for_fake_ip(&fake_ip2),
            Some(domain2.to_string())
        );
    }
}
