// Module: common\serial\string_test.rs
// 1:1 Rust unit test suite corresponding to Go common\serial\string_test.go

#[cfg(test)]
mod tests {
    use super::super::string::*;

    #[test]
    fn test_to_string() {
        let s = "a";
        assert_eq!(to_string_val(&s), "a");
        let string_obj = "a".to_string();
        assert_eq!(to_string_val(&string_obj), "a");
        assert_eq!(to_string_val(&&string_obj), "a");
        let bytes: &[u8] = &[b'b', b'c'];
        assert_eq!(to_string_val(&bytes), "[98 99]");
    }

    #[test]
    fn test_concat() {
        let v1 = "a";
        let v2 = "b";
        let values: Vec<&dyn StringValue> = vec![&v1, &v2];
        assert_eq!(concat(&values), "ab");
        assert_eq!(concat_strings(&["a", "b"]), "ab");
    }

    #[test]
    fn test_concat_and_to_string() {
        let slices = ["hello", " ", "world"];
        assert_eq!(concat_strings(&slices), "hello world");
        assert_eq!(to_string(42), "42");
    }
}
