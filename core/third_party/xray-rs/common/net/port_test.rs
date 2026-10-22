// Module: common\net\port_test.rs
// 1:1 Rust unit test suite corresponding to Go common\net\port_test.go

#[cfg(test)]
mod tests {
    use super::super::{Port, PortList, PortRange};
    use std::str::FromStr;

    #[test]
    fn test_port_parse() {
        let p = Port::from_str("80").unwrap();
        assert_eq!(p.value(), 80);
        assert_eq!(p.to_string(), "80");

        let p_bytes = Port::from_bytes(&[0x01, 0xbb]).unwrap();
        assert_eq!(p_bytes.value(), 443);
    }

    #[test]
    fn test_port_range_and_list() {
        let range = PortRange::new(1000, 2000);
        assert!(range.contains(Port(1000)));
        assert!(range.contains(Port(1500)));
        assert!(range.contains(Port(2000)));
        assert!(!range.contains(Port(999)));
        assert!(!range.contains(Port(2001)));

        let mut list = PortList::new();
        list.add_range(80, 85);
        list.add_single(443);

        assert!(list.contains(Port(80)));
        assert!(list.contains(Port(82)));
        assert!(list.contains(Port(443)));
        assert!(!list.contains(Port(8080)));
    }
}
