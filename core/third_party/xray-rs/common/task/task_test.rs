// Module: common\task\task_test.rs
// 1:1 Rust unit test suite corresponding to Go common\task\task_test.go

#[cfg(test)]
mod tests {
    use super::super::task::{on_success, parallel_run, parallel_run_boxed};
    use crate::common::errors::Error;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    async fn make_task(val: i32) -> i32 {
        val
    }

    #[tokio::test]
    async fn test_parallel_tasks() {
        let tasks = vec![make_task(10), make_task(20)];
        let results = parallel_run(tasks).await;
        assert_eq!(results, vec![10, 20]);
    }

    #[test]
    fn test_on_success() {
        let counter = Arc::new(AtomicU32::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();

        let mut combined = on_success(
            move || {
                c1.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
            move || {
                c2.fetch_add(10, Ordering::SeqCst);
                Ok(())
            },
        );

        assert!(combined().is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 11);

        let mut fail_first = on_success(
            || Err(Error::Config("failed".to_string())),
            || {
                panic!("should not be called");
            },
        );
        assert!(fail_first().is_err());
    }

    #[tokio::test]
    async fn test_run_parallel_success_and_error() {
        let t1 = Box::pin(async { Ok(()) });
        let t2 = Box::pin(async { Ok(()) });
        assert!(parallel_run_boxed(vec![t1, t2]).await.is_ok());

        let f1 = Box::pin(async { Ok(()) });
        let f2 = Box::pin(async { Err(Error::Config("err".to_string())) });
        assert!(parallel_run_boxed(vec![f1, f2]).await.is_err());
    }
}
