// Module: transport\internet\finalmask\noise\noise_test.rs

#[cfg(test)]
mod tests {
    use super::super::config::{NoiseConfig, NoiseItem};
    use super::super::conn::NoisePacketConn;

    #[test]
    fn test_noise_packet_generation_and_delay() {
        let item1 = NoiseItem {
            rand_min: 16,
            rand_max: 32,
            rand_range_min: 10,
            rand_range_max: 50,
            packet: Vec::new(),
            delay_min: 5,
            delay_max: 15,
        };
        let item2 = NoiseItem {
            rand_min: 0,
            rand_max: 0,
            rand_range_min: 0,
            rand_range_max: 255,
            packet: b"fixed-pattern".to_vec(),
            delay_min: 20,
            delay_max: 20,
        };
        let config = NoiseConfig {
            reset_min: 5,
            reset_max: 10,
            items: vec![item1, item2],
            ..Default::default()
        };

        let conn = NoisePacketConn::new(config);
        assert!(conn.should_send_noise("192.168.1.1:8080"));

        let packets = conn.generate_noise_packets();
        assert_eq!(packets.len(), 2);

        // First packet random length 16..=32 and bytes 10..=50
        assert!(packets[0].0.len() >= 16 && packets[0].0.len() <= 32);
        for &b in &packets[0].0 {
            assert!(b >= 10 && b <= 50);
        }
        assert!(packets[0].1.as_millis() >= 5 && packets[0].1.as_millis() <= 15);

        // Second packet fixed
        assert_eq!(packets[1].0, b"fixed-pattern");
        assert_eq!(packets[1].1.as_millis(), 20);

        // Mark sent
        conn.mark_sent("192.168.1.1:8080");
        assert!(!conn.should_send_noise("192.168.1.1:8080"));
        // Different address should still send noise
        assert!(conn.should_send_noise("192.168.1.2:8080"));
    }
}
