#[cfg(test)]
mod tests {
    use crate::app::reverse::{Bridge, Portal, ReverseManager};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_reverse_portal_dispatch_and_pull() {
        let portal = Portal::new("portal-in", "reverse.service.internal");
        let (s1, mut s2) = tokio::io::duplex(1024);

        portal.dispatch(Box::pin(s1)).await.unwrap();

        let mut pulled = portal.pull_stream().await.unwrap();

        // Write from s2 side, read from pulled stream
        s2.write_all(b"Reverse Tunnel Stream Message")
            .await
            .unwrap();

        let mut recv = [0u8; 29];
        pulled.read_exact(&mut recv).await.unwrap();
        assert_eq!(&recv, b"Reverse Tunnel Stream Message");
    }

    #[test]
    fn test_reverse_manager_registration() {
        let mut mgr = ReverseManager::new();
        let portal = Portal::new("portal-tag", "domain.internal");
        let bridge = Bridge::new("bridge-tag", "domain.internal");

        mgr.register_portal(portal);
        mgr.register_bridge(bridge);

        assert!(mgr.get_portal("domain.internal").is_some());
        assert!(mgr.get_portal("unknown.domain").is_none());
    }
}
