// Module: common\errors\errors_test.rs
// 1:1 Rust unit test suite corresponding to Go common\errors\errors_test.go

#[cfg(test)]
mod tests {
    use super::super::errors::{
        cause, get_severity, log_debug, log_info, log_warning, new as new_error,
        new_error as make_error,
    };
    use super::super::feature_errors::{
        missing_feature, print_deprecated_feature_warning,
        print_non_removal_deprecated_feature_warning, print_removed_feature_error,
    };
    use super::super::multi_error::{MultiError, all_equal, combine};
    use crate::common::errors::Error;
    use crate::common::log::Severity;

    #[test]
    fn test_error_severity_propagation() {
        let err = new_error("TestError");
        assert_eq!(get_severity(&err), Severity::Info);

        let err2 = new_error("TestError2").base(new_error("io.EOF"));
        assert_eq!(get_severity(&err2), Severity::Info);

        let err3 = new_error("TestError3")
            .base(new_error("io.EOF"))
            .at_warning();
        assert_eq!(get_severity(&err3), Severity::Warning);

        let err4 = new_error("TestError4")
            .base(new_error("io.EOF"))
            .at_warning();
        let err5 = new_error("TestError5").base(err4);
        assert_eq!(get_severity(&err5), Severity::Warning);
        assert!(err5.to_string().contains("io.EOF"));
    }

    #[test]
    fn test_error_message_chain() {
        let mut err_b = new_error("b");
        err_b.caller = "common/errors_test".into();

        let mut err_a = new_error("a");
        err_a.caller = "common/errors_test".into();
        let err_chain = err_a.base(err_b);

        assert_eq!(
            err_chain.to_string(),
            "common/errors_test: a > common/errors_test: b"
        );
    }

    #[test]
    fn test_error_cause_unwrapping() {
        let root = new_error("root_cause");
        let middle = new_error("middle").base(root);
        let top = new_error("top").base(middle);

        let root_found = cause(&top);
        assert_eq!(root_found.message, "root_cause");
    }

    #[test]
    fn test_multi_error_combine_and_all_equal() {
        let mut me = MultiError::new();
        assert!(me.is_empty());
        assert!(!me.has_errors());

        me.add(Error::Other("timeout error".into()));
        me.add(Error::Other("timeout failure".into()));
        assert_eq!(me.len(), 2);
        assert!(all_equal("timeout", &me));
        assert!(!all_equal("io error", &me));

        let res = me.into_result();
        assert!(res.is_err());

        // Test combine
        let combined = combine(vec![
            Some(Error::Other("first".into())),
            None,
            Some(Error::Other("second".into())),
        ]);
        assert!(combined.is_some());
        let c = combined.unwrap();
        assert_eq!(c.len(), 2);
        assert!(c.to_string().contains("first"));
        assert!(c.to_string().contains("second"));
    }

    #[test]
    fn test_feature_errors() {
        print_non_removal_deprecated_feature_warning("old_mux", "new_mux");
        print_deprecated_feature_warning("feature_x", "feature_y");
        print_deprecated_feature_warning("feature_z", "");

        let err1 = print_removed_feature_error("legacy_kcp", "modern_kcp");
        assert!(
            err1.to_string()
                .contains("removed and migrated to modern_kcp")
        );

        let err2 = print_removed_feature_error("obsolete_crypto", "");
        assert!(err2.to_string().contains("has been removed"));

        let missing = missing_feature("telemetry");
        assert!(missing.to_string().contains("telemetry"));

        let legacy_err = make_error("simple legacy error");
        assert!(legacy_err.to_string().contains("simple legacy error"));

        log_debug("debug message");
        log_info("info message");
        log_warning("warning message");
    }
}
