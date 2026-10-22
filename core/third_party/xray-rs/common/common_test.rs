// Module: common\common_test.rs
// 1:1 Rust unit test suite corresponding to Go common\common_test.go

#[cfg(test)]
mod tests {
    use super::super::common::must;

    #[test]
    fn test_must_success() {
        let res: Result<i32, &str> = Ok(42);
        assert_eq!(must(res), 42);
    }

    #[test]
    #[should_panic]
    fn test_must_panic() {
        let res: Result<i32, &str> = Err("error");
        must(res);
    }
}
