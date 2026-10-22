// Module: app\router\command\command.rs
// 1:1 Rust implementation corresponding to Go app\router\command\command.go

use crate::app::commander::Service;
use crate::app::router::Router;
use crate::common::protocol::SessionContext;
use crate::features::routing::RouterFeature;
use std::sync::Arc;

pub struct RoutingService {
    router: Arc<Router>,
}

impl RoutingService {
    pub fn new(router: Arc<Router>) -> Self {
        Self { router }
    }

    pub fn test_route(&self, session: &SessionContext) -> Option<String> {
        self.router.pick_outbound(session).map(|s| s.to_string())
    }
}

impl Service for RoutingService {
    fn service_name(&self) -> &str {
        "xray.core.app.router.command.RoutingService"
    }
}
