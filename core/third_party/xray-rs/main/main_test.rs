// Module: main\\main_test.rs
// 1:1 Rust unit test suite corresponding to Go main\\main_test.go

#[cfg(test)]
mod tests {
    use super::super::version::version;

    #[test]
    fn test_main_version() {
        assert_eq!(version(), "26.3.27");
    }
}
