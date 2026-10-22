#[cfg(test)]
mod tests {
    use crate::common::signal::{ActivityTimer, Done, Notifier, Semaphore};
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_done_signal_wait_and_close() {
        let done = Done::new();
        assert!(!done.is_done());

        let done_clone = done.clone();
        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            done_clone.close();
        });

        done.wait().await;
        assert!(done.is_done());
    }

    #[tokio::test]
    async fn test_semaphore_permits() {
        let sem = Semaphore::new(2);
        assert_eq!(sem.available_permits(), 2);

        sem.acquire().await;
        assert_eq!(sem.available_permits(), 1);

        sem.signal();
        assert_eq!(sem.available_permits(), 2);

        sem.add_permits(2);
        assert_eq!(sem.available_permits(), 4);
    }

    #[tokio::test]
    async fn test_notifier_broadcast() {
        let notifier = Notifier::new();
        let mut rx1 = notifier.subscribe();
        let mut rx2 = notifier.subscribe();

        notifier.notify();

        assert!(rx1.recv().await.is_ok());
        assert!(rx2.recv().await.is_ok());
    }

    #[tokio::test]
    async fn test_activity_timer_update_and_timeout() {
        let timer = ActivityTimer::new(Duration::from_millis(100));
        assert!(!timer.is_timed_out());

        sleep(Duration::from_millis(50)).await;
        timer.update();
        sleep(Duration::from_millis(60)).await;
        assert!(!timer.is_timed_out());

        sleep(Duration::from_millis(110)).await;
        assert!(timer.is_timed_out());
    }
}
