// Module: common\peer\peer_test.rs

#[cfg(test)]
mod tests {
    use super::super::latency::{AverageLatency, HasLatency};
    use super::super::peer::Peer;

    #[test]
    fn test_average_latency_calculation() {
        let latency = AverageLatency::new(0);
        assert_eq!(latency.value(), 0);

        // update with 30: (0 + 30*2) / 3 = 20
        latency.update(30);
        assert_eq!(latency.value(), 20);

        // update with 50: (20 + 50*2) / 3 = 40
        latency.update(50);
        assert_eq!(latency.value(), 40);
    }

    #[test]
    fn test_peer_latency_interface() {
        let peer = Peer::new();
        peer.conn_latency.update(60);
        peer.handshake_latency.update(90);

        assert_eq!(peer.connection_latency().value(), 40);
        assert_eq!(peer.handshake_latency().value(), 60);
    }
}
