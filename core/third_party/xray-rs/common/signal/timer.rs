// Module: common\signal\timer.rs
// 1:1 Rust implementation corresponding to Go common\signal\timer.go

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;

pub trait ActivityUpdater: Send + Sync {
    fn update(&self);
}

pub struct ActivityTimer {
    timeout: Mutex<Duration>,
    last_activity: Arc<AtomicU64>,
    start_time: Instant,
    consumed: Arc<AtomicBool>,
    on_timeout: Arc<Mutex<Option<Box<dyn FnOnce() + Send + 'static>>>>,
    task_handle: Mutex<Option<JoinHandle<()>>>,
}

impl ActivityTimer {
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout: Mutex::new(timeout),
            last_activity: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
            consumed: Arc::new(AtomicBool::new(false)),
            on_timeout: Arc::new(Mutex::new(None)),
            task_handle: Mutex::new(None),
        }
    }

    pub fn with_callback<F>(timeout: Duration, on_timeout: F) -> Arc<Self>
    where
        F: FnOnce() + Send + 'static,
    {
        let timer = Arc::new(Self {
            timeout: Mutex::new(timeout),
            last_activity: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
            consumed: Arc::new(AtomicBool::new(false)),
            on_timeout: Arc::new(Mutex::new(Some(Box::new(on_timeout)))),
            task_handle: Mutex::new(None),
        });
        timer.set_timeout(timeout);
        timer
    }

    pub fn update(&self) {
        let elapsed = self.start_time.elapsed().as_millis() as u64;
        self.last_activity.store(elapsed, Ordering::SeqCst);
    }

    pub fn is_timeout(&self) -> bool {
        let last = self.last_activity.load(Ordering::SeqCst);
        let current = self.start_time.elapsed().as_millis() as u64;
        let timeout_millis = self.timeout.lock().unwrap().as_millis() as u64;
        current.saturating_sub(last) >= timeout_millis
    }

    pub fn is_timed_out(&self) -> bool {
        self.is_timeout()
    }

    pub fn finish(&self) {
        if !self.consumed.swap(true, Ordering::SeqCst)
            && let Some(cb) = self.on_timeout.lock().unwrap().take()
        {
            cb();
        }
    }

    pub fn set_timeout(self: &Arc<Self>, timeout: Duration) {
        if self.consumed.load(Ordering::Relaxed) {
            return;
        }

        if timeout.is_zero() {
            self.finish();
            return;
        }

        *self.timeout.lock().unwrap() = timeout;

        // Cancel previous background checker task
        let mut handle_guard = self.task_handle.lock().unwrap();
        if let Some(handle) = handle_guard.take() {
            handle.abort();
        }

        self.update();
        let this = Arc::downgrade(self);
        let new_handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(timeout).await;
                if let Some(timer) = this.upgrade() {
                    if timer.consumed.load(Ordering::Relaxed) {
                        break;
                    }
                    if timer.is_timeout() {
                        timer.finish();
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        *handle_guard = Some(new_handle);
    }
}

impl ActivityUpdater for ActivityTimer {
    fn update(&self) {
        self.update();
    }
}

impl Drop for ActivityTimer {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.task_handle.lock()
            && let Some(handle) = guard.take()
        {
            handle.abort();
        }
    }
}

pub fn cancel_after_inactivity<F>(on_timeout: F, timeout: Duration) -> Arc<ActivityTimer>
where
    F: FnOnce() + Send + 'static,
{
    ActivityTimer::with_callback(timeout, on_timeout)
}
