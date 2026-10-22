// Module: common\type_test.rs
// 1:1 Rust unit test suite corresponding to Go common\type_test.go

#[cfg(test)]
mod tests {
    use super::super::r#type::type_of;

    #[test]
    fn test_type_of() {
        let x = 42u64;
        assert_eq!(type_of(&x), "u64");
    }
}
