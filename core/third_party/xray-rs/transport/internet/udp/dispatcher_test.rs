// Module: transport\internet\udp\dispatcher_test.rs
// 1:1 Rust implementation corresponding to Go transport\internet\udp\dispatcher_test.go

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;
    use tokio::time::sleep;

    use crate::common::buf::Buffer;
    use crate::common::errors::Result;
    use crate::common::net::{Address, Destination, Port};
    use crate::common::protocol::udp::UdpPacket;
    use crate::transport::internet::udp::dispatcher::{
        Dispatcher, LinkDispatcher, ResponseCallback, dial_dispatcher,
    };
    use crate::transport::internet::udp::hub::{Hub, HubOption};
    use crate::transport::link::Link;
    use crate::transport::pipe::{new_with_options, with_size_limit};

    struct TestDispatcher {
        count: Arc<AtomicU32>,
        downlink_reader: crate::transport::pipe::Reader,
        uplink_writer: crate::transport::pipe::Writer,
    }

    #[async_trait::async_trait]
    impl LinkDispatcher for TestDispatcher {
        async fn dispatch(&self, _dest: Destination) -> Result<Link> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(Link::new(
                self.downlink_reader.clone(),
                self.uplink_writer.clone(),
            ))
        }
    }

    #[tokio::test]
    async fn test_same_destination_dispatching() {
        let (uplink_reader, uplink_writer) = new_with_options(vec![with_size_limit(1024)]);
        let (downlink_reader, downlink_writer) = new_with_options(vec![with_size_limit(1024)]);

        // Background echo loop: reads from uplink_reader and writes to downlink_writer
        tokio::spawn(async move {
            loop {
                match uplink_reader.read_multi_buffer().await {
                    Ok(mb) => {
                        if mb.is_empty() {
                            break;
                        }
                        if downlink_writer.write_multi_buffer(mb).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let count = Arc::new(AtomicU32::new(0));
        let td = Arc::new(TestDispatcher {
            count: count.clone(),
            downlink_reader,
            uplink_writer,
        });

        let dest = Destination::udp(Address::ip([127, 0, 0, 1].into()), 53);

        let msg_count = Arc::new(AtomicU32::new(0));
        let msg_count_clone = msg_count.clone();

        let callback: ResponseCallback = Arc::new(move |_packet: &UdpPacket| {
            msg_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        let dispatcher = Dispatcher::new(td, callback);

        for _ in 0..6 {
            let mut b = Buffer::new();
            b.write(b"abcd").unwrap();
            dispatcher.dispatch(dest.clone(), b).await.unwrap();
        }

        sleep(Duration::from_millis(150)).await;

        assert_eq!(
            count.load(Ordering::SeqCst),
            1,
            "Only 1 connection should be dispatched for same dest"
        );
        assert_eq!(
            msg_count.load(Ordering::SeqCst),
            6,
            "All 6 messages should be received by callback"
        );

        dispatcher.remove_ray().await;
    }

    #[tokio::test]
    async fn test_dial_dispatcher() {
        let (uplink_reader, uplink_writer) = new_with_options(vec![with_size_limit(1024)]);
        let (downlink_reader, downlink_writer) = new_with_options(vec![with_size_limit(1024)]);

        tokio::spawn(async move {
            loop {
                match uplink_reader.read_multi_buffer().await {
                    Ok(mb) => {
                        if mb.is_empty() {
                            break;
                        }
                        if downlink_writer.write_multi_buffer(mb).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let count = Arc::new(AtomicU32::new(0));
        let td = Arc::new(TestDispatcher {
            count: count.clone(),
            downlink_reader,
            uplink_writer,
        });

        let conn = dial_dispatcher(td).await.unwrap();
        let dest = Destination::udp(Address::ip([127, 0, 0, 1].into()), 1053);

        let payload = b"hello packet";
        let sent = conn.write_to(payload, dest.clone()).await.unwrap();
        assert_eq!(sent, payload.len());

        let mut recv_buf = vec![0u8; 128];
        let (n, src) = conn.read_from(&mut recv_buf).await.unwrap();
        assert_eq!(n, payload.len());
        assert_eq!(&recv_buf[..n], payload);
        assert_eq!(src, dest);

        conn.close().await;
    }

    #[tokio::test]
    async fn test_hub_options_and_bind() {
        let address = Address::ip([127, 0, 0, 1].into());
        let hub = Hub::listen_udp(
            &address,
            Port::new(0),
            None,
            &[
                HubOption::Capacity(128),
                HubOption::ReceiveOriginalDestination(false),
            ],
        )
        .await
        .unwrap();

        assert_eq!(hub.capacity(), 128);
        assert!(!hub.receive_original_destination());
        assert!(hub.local_addr().is_ok());

        hub.close();
    }
}
