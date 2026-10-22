#[cfg(test)]
mod tests {
    use crate::common::task::{Periodic, parallel_run_boxed};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[tokio::test]
    async fn test_periodic_task_execution_and_cancellation() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let mut periodic = Periodic::new(Duration::from_millis(10));
        periodic.start(move || {
            let c = counter_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        });

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(counter.load(Ordering::SeqCst) >= 2);

        periodic.close();
        let val_after_close = counter.load(Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(30)).await;
        assert_eq!(counter.load(Ordering::SeqCst), val_after_close);
    }

    #[tokio::test]
    async fn test_parallel_run_success() {
        use crate::common::errors::Result;
        use std::future::Future;
        use std::pin::Pin;

        let task1: Pin<Box<dyn Future<Output = Result<()>> + Send>> = Box::pin(async { Ok(()) });
        let task2: Pin<Box<dyn Future<Output = Result<()>> + Send>> = Box::pin(async { Ok(()) });
        let task3: Pin<Box<dyn Future<Output = Result<()>> + Send>> = Box::pin(async { Ok(()) });

        let res = parallel_run_boxed(vec![task1, task2, task3]).await;
        assert!(res.is_ok());
    }
}
