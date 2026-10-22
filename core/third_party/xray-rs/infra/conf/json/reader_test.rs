// Module: infra\conf\json\reader_test.rs
// 1:1 Rust unit test suite corresponding to Go infra\conf\json\reader_test.go

#[cfg(test)]
mod tests {
    use super::super::reader::parse_json_value;

    #[test]
    fn test_json_reader() {
        let v = parse_json_value(r#"{"key": "value"}"#).unwrap();
        assert_eq!(v["key"], "value");
    }
}
