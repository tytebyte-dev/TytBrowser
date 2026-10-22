// Module: testing\\scenarios\\common_coverage.rs
#[cfg(test)]
mod tests {
    use crate::common::units::format_bytesize;

    #[test]
    fn test_coverage_units() {
        assert_eq!(format_bytesize(1024), "1.00KB");
    }
}
