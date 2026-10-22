// Module: common\task\task.rs
// 1:1 Rust implementation corresponding to Go common\task\task.go

use crate::common::errors::{Error, Result};
use futures::future::join_all;
use std::future::Future;
use std::pin::Pin;

/// OnSuccess executes g() after f() returns Ok(()).
pub fn on_success<F, G>(mut f: F, mut g: G) -> impl FnMut() -> Result<()>
where
    F: FnMut() -> Result<()>,
    G: FnMut() -> Result<()>,
{
    move || {
        f()?;
        g()
    }
}

pub async fn parallel_run_boxed(
    tasks: Vec<Pin<Box<dyn Future<Output = Result<()>> + Send>>>,
) -> Result<()> {
    let handles: Vec<_> = tasks.into_iter().map(tokio::spawn).collect();
    for handle in handles {
        match handle.await {
            Ok(Ok(())) => continue,
            Ok(Err(e)) => return Err(e),
            Err(e) => {
                return Err(Error::Io(std::io::Error::other(e.to_string())));
            }
        }
    }
    Ok(())
}

pub async fn parallel_run<F, T>(tasks: Vec<F>) -> Vec<T>
where
    F: Future<Output = T>,
{
    join_all(tasks).await
}
