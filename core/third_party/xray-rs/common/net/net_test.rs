// Module: common\net\net_test.rs
#[cfg(test)]
mod tests {
    use super::super::Destination;
    use super::super::destination_compact::to_compact_string;

    #[test]
    fn test_compact_destination() {
        let dest = Destination::ip("127.0.0.1", 8080).unwrap();
        assert_eq!(to_compact_string(&dest), "127.0.0.1:8080");
    }
}
