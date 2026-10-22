// Module: common\drain\drain_test.rs

#[cfg(test)]
mod tests {
    use super::super::drain::*;
    use super::super::drainer::*;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_drain_read_n() {
        let (mut client, mut server) = tokio::io::duplex(1024);
        tokio::spawn(async move {
            client.write_all(b"0123456789").await.unwrap();
        });

        let drained = drain_read_n(&mut server, 5).await.unwrap();
        assert_eq!(drained, 5);
    }

    #[tokio::test]
    async fn test_behavior_seed_limited_drainer() {
        let mut drainer = BehaviorSeedLimitedDrainer::new(12345, 100, 50, 20);
        assert!(drainer.drain_size >= 100);

        let initial_size = drainer.drain_size;
        drainer.acknowledge_receive(30);
        assert_eq!(drainer.drain_size, initial_size - 30);

        drainer.acknowledge_receive(1000);
        assert_eq!(drainer.drain_size, 0);

        let nop = NopDrainer::new();
        let (client, mut server) = tokio::io::duplex(64);
        drop(client);
        assert!(nop.drain(&mut server).await.is_ok());
    }
}
