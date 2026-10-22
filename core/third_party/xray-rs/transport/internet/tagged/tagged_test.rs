// Module: transport\internet\tagged\tagged_test.rs
// 1:1 Rust unit test for tagged and taggedimpl

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use crate::common::errors::Result;
    use crate::common::net::{Address, BoxStream, Destination};
    use crate::common::protocol::SessionContext;
    use crate::features::feature::Feature;
    use crate::features::routing::Dispatcher;
    use crate::transport::internet::tagged::dial;
    use crate::transport::internet::tagged::taggedimpl::{
        dial_tagged_outbound, register_tagged_dialer,
    };

    struct MockDispatcher;

    impl Feature for MockDispatcher {
        fn feature_type(&self) -> &'static str {
            "mock_dispatcher"
        }
    }

    #[async_trait::async_trait]
    impl Dispatcher for MockDispatcher {
        async fn dispatch(&self, session: SessionContext, mut stream: BoxStream) -> Result<()> {
            let mut buf = vec![0u8; 100];
            let n = stream
                .read(&mut buf)
                .await
                .map_err(crate::common::errors::Error::Io)?;
            assert_eq!(&buf[..n], b"request from tagged dial");
            let response = format!(
                "echo to {}:{}",
                session.inbound_tag, session.destination.port
            );
            stream
                .write_all(response.as_bytes())
                .await
                .map_err(crate::common::errors::Error::Io)?;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_dial_tagged_outbound_direct() {
        let dispatcher = Arc::new(MockDispatcher);
        let dest = Destination::tcp(Address::ip([127, 0, 0, 1].into()), 8080);

        let mut client_stream = dial_tagged_outbound(Some(dispatcher), dest, "out-direct".into())
            .await
            .unwrap();

        client_stream
            .write_all(b"request from tagged dial")
            .await
            .unwrap();

        let mut resp = vec![0u8; 128];
        let n = client_stream.read(&mut resp).await.unwrap();
        assert_eq!(&resp[..n], b"echo to out-direct:8080");
    }

    #[tokio::test]
    async fn test_registered_tagged_dialer() {
        register_tagged_dialer().await;

        let dispatcher = Arc::new(MockDispatcher);
        let dest = Destination::tcp(Address::ip([127, 0, 0, 1].into()), 9090);

        let mut stream = dial(Some(dispatcher), dest, "tag-proxy".into())
            .await
            .unwrap();

        stream.write_all(b"request from tagged dial").await.unwrap();

        let mut resp = vec![0u8; 128];
        let n = stream.read(&mut resp).await.unwrap();
        assert_eq!(&resp[..n], b"echo to tag-proxy:9090");
    }
}
