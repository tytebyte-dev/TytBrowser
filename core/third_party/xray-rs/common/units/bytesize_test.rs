// Module: common\units\bytesize_test.rs
// 1:1 Rust unit test suite corresponding to Go common\units\bytesize_test.go

#[cfg(test)]
mod tests {
    use super::super::bytesize::*;

    #[test]
    fn test_byte_sizes() {
        let mut size = ByteSize(0);
        assert_size_string(size, "0");
        size.0 += 1;
        assert_size_value(assert_size_string(size, "1.00B"), size);

        size.0 <<= 10;
        assert_size_value(assert_size_string(size, "1.00KB"), size);

        size.0 <<= 10;
        assert_size_value(assert_size_string(size, "1.00MB"), size);

        size.0 <<= 10;
        assert_size_value(assert_size_string(size, "1.00GB"), size);

        size.0 <<= 10;
        assert_size_value(assert_size_string(size, "1.00TB"), size);

        size.0 <<= 10;
        assert_size_value(assert_size_string(size, "1.00PB"), size);

        size.0 <<= 10;
        assert_size_value(assert_size_string(size, "1.00EB"), size);
    }

    fn assert_size_value(size_str: &str, expected: ByteSize) {
        let mut actual = ByteSize(0);
        actual.parse(size_str).expect("parse failed");
        assert_eq!(
            actual, expected,
            "expect {:?}, but got {:?}",
            expected, actual
        );
    }

    fn assert_size_string(size: ByteSize, expected: &str) -> &'static str {
        let actual = size.to_string();
        assert_eq!(actual, expected, "expect {}, but got {}", expected, actual);
        // Leak is fine in test to return &'static str matching Go signature
        Box::leak(expected.to_string().into_boxed_str())
    }
}
