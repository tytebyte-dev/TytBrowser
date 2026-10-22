#[cfg(test)]
mod tests {
    use crate::common::bytespool::{alloc, free};
    use crate::common::errors::{Error, Result};
    use crate::common::retry::RetryStrategy;
    use crate::common::units::{GB, KB, MB, format_bytesize, parse_bytesize};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[test]
    fn test_bytespool_alloc_and_free() {
        let buf = alloc(1024);
        assert_eq!(buf.len(), 1024);
        free(buf);

        let buf2 = alloc(8000);
        assert_eq!(buf2.len(), 8000);
        free(buf2);
    }

    #[test]
    fn test_units_parse_and_format() {
        assert_eq!(parse_bytesize("10KB").unwrap(), 10 * KB);
        assert_eq!(parse_bytesize("2.5MB").unwrap(), (2.5 * MB as f64) as u64);
        assert_eq!(parse_bytesize("1GB").unwrap(), GB);
        assert_eq!(parse_bytesize("512B").unwrap(), 512);

        assert_eq!(format_bytesize(0), "0");
        assert_eq!(format_bytesize(500), "500.00B");
        assert_eq!(format_bytesize(1024), "1.00KB");
        assert_eq!(format_bytesize(1024 * 1024 * 5), "5.00MB");
    }

    #[tokio::test]
    async fn test_retry_strategy_success_after_failure() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_clone = attempts.clone();

        let strategy = RetryStrategy::timed(3, Duration::from_millis(10));
        let res: Result<i32> = strategy
            .run(|| {
                let attempts_clone = attempts_clone.clone();
                async move {
                    let count = attempts_clone.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(Error::Other("temporary error".into()))
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;

        assert_eq!(res.unwrap(), 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }
}
