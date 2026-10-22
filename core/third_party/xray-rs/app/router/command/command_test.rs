// Module: app\router\command\command_test.rs
// 1:1 Rust unit test suite corresponding to Go app\router\command\command_test.go

#[cfg(test)]
mod tests {
    use super::super::command::RoutingService;
    use crate::app::commander::Service;
    use crate::app::router::Router;
    use std::sync::Arc;

    #[test]
    fn test_routing_service_registration() {
        let router = Arc::new(Router::new(Vec::new(), None));
        let service = RoutingService::new(router);
        assert_eq!(
            service.service_name(),
            "xray.core.app.router.command.RoutingService"
        );
    }
}
