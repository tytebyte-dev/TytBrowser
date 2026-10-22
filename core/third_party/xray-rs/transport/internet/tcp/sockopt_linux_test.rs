// Module: transport\internet\tcp\sockopt_linux_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\tcp\sockopt_linux_test.go

#[cfg(test)]
mod tests {
    use super::super::sockopt_linux::is_fastopen_supported;

    #[test]
    fn test_linux_fastopen_support() {
        assert!(is_fastopen_supported());
    }
}
