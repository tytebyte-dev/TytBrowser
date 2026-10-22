// Module: common\retry\retry_test.rs
// 1:1 Rust unit test suite corresponding to Go common\retry\retry_test.go

#[cfg(test)]
mod tests {
    use super::super::retry::*;
    use crate::common::errors::{Error, Result};
    use std::time::Instant;

    fn error_test_only() -> Error {
        Error::Other("this is a fake error".into())
    }

    #[test]
    fn test_no_retry() {
        let err = timed(10, 1000).on(|| Ok(()));
        assert!(err.is_ok());
    }

    #[test]
    fn test_retry_once() {
        let mut called = 0;
        let err = timed(10, 1).on(|| {
            if called == 0 {
                called += 1;
                Err(error_test_only())
            } else {
                Ok(())
            }
        });
        assert!(err.is_ok());
        assert_eq!(called, 1);
    }

    #[test]
    fn test_retry_multiple() {
        let mut called = 0;
        let err = timed(10, 1).on(|| {
            if called < 5 {
                called += 1;
                Err(error_test_only())
            } else {
                Ok(())
            }
        });
        assert!(err.is_ok());
        assert_eq!(called, 5);
    }

    #[test]
    fn test_retry_exhausted() {
        let mut called = 0;
        let err = timed(2, 1).on(|| {
            called += 1;
            Err(error_test_only())
        });
        assert!(err.is_err());
        assert_eq!(called, 2);
        assert!(err.unwrap_err().to_string().contains(ERR_RETRY_FAILED));
    }

    #[tokio::test]
    async fn test_exponential_backoff_async() {
        let start = Instant::now();
        let mut called = 0;
        let res: Result<i32> = exponential_backoff(3, 2)
            .on_async(|| {
                called += 1;
                async move {
                    if called < 3 {
                        Err(error_test_only())
                    } else {
                        Ok(100)
                    }
                }
            })
            .await;
        assert_eq!(res.unwrap(), 100);
        assert!(start.elapsed().as_millis() >= 2);
    }
}
