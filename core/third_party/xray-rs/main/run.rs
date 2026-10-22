// Module: main\run.rs
// 1:1 Rust implementation corresponding to Go main\run.go

use crate::common::errors::Result;
use crate::core::Instance;
use crate::infra::conf::Config;

pub async fn run_server(config: Config) -> Result<()> {
    let instance = Instance::from_config(config)?;
    let handles = instance.start().await?;
    for h in handles {
        let _ = h.await;
    }
    Ok(())
}
