// Module: common\task\periodic_test.rs
// 1:1 Rust unit test suite corresponding to Go common\task\periodic_test.go

#[cfg(test)]
mod tests {
    use super::super::periodic::Periodic;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_periodic_task_run_and_close() {
        let counter = Arc::new(AtomicU32::new(0));
        let c_clone = counter.clone();
        let mut p = Periodic::new(Duration::from_millis(20));
        p.start(move || {
            let c = c_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        });
        sleep(Duration::from_millis(150)).await;
        p.close();
        let val = counter.load(Ordering::SeqCst);
        assert!(val >= 1);
    }

    #[tokio::test]
    async fn test_periodic_task_stop_sync_style() {
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();
        let mut p = Periodic::with_execute(Duration::from_millis(30), move || {
            c.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        p.start_execute().unwrap();
        // Immediately executes on start, so count >= 1
        assert!(counter.load(Ordering::SeqCst) >= 1);

        sleep(Duration::from_millis(70)).await;
        p.close();
        assert!(p.has_closed());
    }
}
