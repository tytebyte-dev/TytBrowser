// Module: common\protocol\quic\sniff_test.rs
// 1:1 Rust unit test suite corresponding to Go common\protocol\quic\sniff_test.go

#[cfg(test)]
mod tests {
    use super::super::sniff::sniff_quic;

    #[test]
    fn test_quic_sniff_empty() {
        assert!(sniff_quic(&[]).is_err());
    }
}
