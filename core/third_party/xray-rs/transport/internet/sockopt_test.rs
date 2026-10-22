// Module: transport\internet\sockopt_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\sockopt_test.go

#[cfg(test)]
mod tests {
    use super::super::sockopt::{SocketOptions, is_tcp_socket, is_udp_socket};

    #[test]
    fn test_socket_options_builder_and_tfo() {
        let opts = SocketOptions::new().with_mark(100).with_tfo(256);
        assert_eq!(opts.mark, 100);
        assert_eq!(opts.tfo, 256);
        assert_eq!(opts.parse_tfo_value(), 256);

        let default_opts = SocketOptions::default();
        assert_eq!(default_opts.parse_tfo_value(), -1);

        let neg_opts = SocketOptions::new().with_tfo(-5);
        assert_eq!(neg_opts.parse_tfo_value(), 0);
    }

    #[test]
    fn test_socket_network_check() {
        assert!(is_tcp_socket("tcp"));
        assert!(is_tcp_socket("tcp4"));
        assert!(is_tcp_socket("tcp6"));
        assert!(!is_tcp_socket("udp"));

        assert!(is_udp_socket("udp"));
        assert!(is_udp_socket("udp4"));
        assert!(is_udp_socket("udp6"));
        assert!(!is_udp_socket("tcp"));
    }
}
