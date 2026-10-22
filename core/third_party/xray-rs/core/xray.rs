// Module: core\xray.rs
// 1:1 Rust implementation corresponding to Go core\xray.go

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::task::JoinHandle;

use crate::app::dispatcher::DefaultDispatcher;
use crate::app::proxyman::InboundManager;
use crate::app::stats::StatsManager;
use crate::common::errors::Result;
use crate::infra::conf::Config;
use crate::proxy::tun::WindowsTunDevice;

pub trait Server: Send + Sync {
    fn start(&self) -> Result<Vec<JoinHandle<()>>>;
    fn close(&self) -> Result<()>;
}

pub fn server_type() -> &'static str {
    "xray.core.Instance"
}

pub struct Instance {
    pub dispatcher: Arc<DefaultDispatcher>,
    pub inbound_manager: InboundManager,
    pub tun_devices: Vec<Arc<WindowsTunDevice>>,
    pub stats_manager: Arc<StatsManager>,
    pub running: Arc<AtomicBool>,
}

impl Instance {
    pub fn new(config: Config) -> Result<Self> {
        Self::from_config(config)
    }

    pub fn from_config(config: Config) -> Result<Self> {
        let built = config.build()?;
        let dispatcher = Arc::new(DefaultDispatcher::new(
            built.outbounds,
            Arc::new(built.router),
        ));
        let inbound_manager = InboundManager::new(built.inbounds);
        let tun_devices = built.tun_devices;
        let stats_manager = Arc::new(StatsManager::new());
        let running = Arc::new(AtomicBool::new(false));

        Ok(Self {
            dispatcher,
            inbound_manager,
            tun_devices,
            stats_manager,
            running,
        })
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn type_name(&self) -> &'static str {
        server_type()
    }

    pub async fn start(&self) -> Result<Vec<JoinHandle<()>>> {
        self.running.store(true, Ordering::SeqCst);
        crate::proxy::tun::init_net_config(None).await;
        let mut tasks = self.inbound_manager.start(self.dispatcher.clone()).await?;
        for dev in &self.tun_devices {
            let task = dev.start(self.dispatcher.clone()).await?;
            tasks.push(task);
        }
        Ok(tasks)
    }

    pub fn close(&self) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);
        for dev in &self.tun_devices {
            dev.shutdown();
        }
        Ok(())
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
