// Module: common\signal\timer_test.rs
// 1:1 Rust unit test suite corresponding to Go common\signal\timer_test.go

#[cfg(test)]
mod tests {
    use super::super::timer::{ActivityTimer, cancel_after_inactivity};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    #[tokio::test]
    async fn test_activity_timer_flow() {
        let timer = ActivityTimer::new(Duration::from_millis(50));
        timer.update();
        assert!(!timer.is_timeout());

        tokio::time::sleep(Duration::from_millis(70)).await;
        assert!(timer.is_timeout());

        timer.update();
        assert!(!timer.is_timeout());
    }

    #[tokio::test]
    async fn test_activity_timer_cancel() {
        let canceled = Arc::new(AtomicBool::new(false));
        let c = canceled.clone();

        let _timer = cancel_after_inactivity(
            move || {
                c.store(true, Ordering::SeqCst);
            },
            Duration::from_millis(50),
        );

        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(canceled.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_activity_timer_zero_timeout() {
        let canceled = Arc::new(AtomicBool::new(false));
        let c = canceled.clone();

        let _timer = cancel_after_inactivity(
            move || {
                c.store(true, Ordering::SeqCst);
            },
            Duration::ZERO,
        );

        assert!(canceled.load(Ordering::SeqCst));
    }
}
