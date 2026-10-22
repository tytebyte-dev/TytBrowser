// Module: transport\internet\sockopt_linux_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\sockopt_linux_test.go

#[cfg(test)]
mod tests {
    use super::super::sockopt::SocketOptions;

    #[test]
    fn test_linux_sockopt_builder() {
        let opt = SocketOptions::new().with_mark(255).with_tfo(1);
        assert_eq!(opt.mark, 255);
        assert_eq!(opt.tfo, 1);
        assert_eq!(opt.parse_tfo_value(), 1);
    }
}
