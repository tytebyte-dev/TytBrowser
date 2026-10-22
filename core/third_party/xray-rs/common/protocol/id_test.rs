// Module: common\protocol\id_test.rs
// 1:1 Rust unit test suite corresponding to Go common\protocol\id_test.go

#[cfg(test)]
mod tests {
    use super::super::id::Id;

    #[test]
    fn test_id_equals() {
        let raw = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let id1 = Id::new(uuid::Uuid::from_bytes(raw));
        let id2 = Id::new(uuid::Uuid::from_bytes(raw));
        assert_eq!(id1, id2);
        assert_eq!(id1.to_string(), id2.to_string());
    }
}
