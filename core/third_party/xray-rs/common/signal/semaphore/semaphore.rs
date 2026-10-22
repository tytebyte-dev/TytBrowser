// Module: common\signal\semaphore\semaphore.rs
// 1:1 Rust implementation corresponding to Go common\signal\semaphore\semaphore.go

use std::sync::Arc;
use tokio::sync::Semaphore as TokioSemaphore;

#[derive(Clone)]
pub struct Instance {
    sem: Arc<TokioSemaphore>,
}

impl Instance {
    pub fn new(n: usize) -> Self {
        Self {
            sem: Arc::new(TokioSemaphore::new(n.max(1))),
        }
    }

    pub async fn wait(&self) {
        let permit = self.sem.acquire().await.unwrap();
        permit.forget();
    }

    pub async fn acquire(&self) {
        self.wait().await;
    }

    pub fn signal(&self) {
        self.sem.add_permits(1);
    }

    pub fn available_permits(&self) -> usize {
        self.sem.available_permits()
    }

    pub fn add_permits(&self, n: usize) {
        self.sem.add_permits(n);
    }
}

pub type Semaphore = Instance;
