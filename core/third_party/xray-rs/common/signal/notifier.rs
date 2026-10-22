// Module: common\signal\notifier.rs
// 1:1 Rust implementation corresponding to Go common\signal\notifier.go

use std::sync::Arc;
use tokio::sync::Notify;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct Notifier {
    notify: Arc<Notify>,
    broadcast: broadcast::Sender<()>,
}

impl Notifier {
    pub fn new() -> Self {
        let (broadcast, _) = broadcast::channel(16);
        Self {
            notify: Arc::new(Notify::new()),
            broadcast,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let (broadcast, _) = broadcast::channel(capacity.max(1));
        Self {
            notify: Arc::new(Notify::new()),
            broadcast,
        }
    }

    pub fn signal(&self) {
        self.notify.notify_one();
        let _ = self.broadcast.send(());
    }

    pub fn notify(&self) -> usize {
        self.signal();
        self.broadcast.receiver_count()
    }

    pub async fn wait(&self) {
        self.notify.notified().await;
    }

    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.broadcast.subscribe()
    }
}

impl Default for Notifier {
    fn default() -> Self {
        Self::new()
    }
}
