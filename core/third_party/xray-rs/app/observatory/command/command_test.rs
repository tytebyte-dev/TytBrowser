// Module: app\observatory\command\command_test.rs
// 1:1 Rust unit test suite corresponding to Go app\observatory\command

#[cfg(test)]
mod tests {
    use super::super::command::ObservatoryCommandServer;
    use super::super::command_grpc_pb::ObservatoryService;
    use super::super::command_pb::GetOutboundStatusRequest;
    use crate::app::observatory::Observatory;
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn test_observatory_command() {
        let obs = Arc::new(Observatory::new(
            "http://example.com",
            Duration::from_secs(5),
            vec![],
        ));
        obs.record_status("out-1", true, 50, None);
        let srv = ObservatoryCommandServer::new(obs);
        assert_eq!(srv.get_outbound_status("out-1"), Some(true));
        assert_eq!(srv.get_outbound_status("unknown"), None);

        let resp = <ObservatoryCommandServer as ObservatoryService>::get_outbound_status(
            &srv,
            GetOutboundStatusRequest {},
        )
        .await
        .expect("grpc response");
        let observation = resp.status.expect("observation status");
        assert_eq!(observation.status.len(), 1);
        assert_eq!(observation.status[0].outbound_tag, "out-1");
        assert!(observation.status[0].alive);
        assert_eq!(observation.status[0].delay, 50);
    }
}
