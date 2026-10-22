// Module: infra\conf\common_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\common_test.go

#[cfg(test)]
mod tests {
    use super::super::common::StringList;

    #[test]
    fn test_string_list() {
        let list = StringList(vec!["a".into(), "b".into()]);
        assert_eq!(list.0.len(), 2);
    }
}
