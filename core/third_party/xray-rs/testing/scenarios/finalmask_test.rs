#[cfg(test)]
mod tests {
    use crate::transport::internet::finalmask::{
        FragmentConfig, Fragmenter, NoiseConfig, NoiseGenerator,
    };
    use std::time::Duration;

    #[tokio::test]
    async fn test_finalmask_fragment_and_noise() {
        let fragmenter = Fragmenter::new(FragmentConfig {
            min_len: 10,
            max_len: 20,
            min_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(5),
            packets: "tlshello".into(),
        });

        let mut fake_tls_hello = vec![0x16, 0x03, 0x01, 0x00, 0x50, 0x01];
        fake_tls_hello.extend_from_slice(&[0xaa; 75]);

        let mut output = Vec::new();
        fragmenter
            .write_fragmented(&mut output, &fake_tls_hello)
            .await
            .unwrap();
        assert_eq!(output, fake_tls_hello);

        let noise_gen = NoiseGenerator::new(NoiseConfig {
            min_len: 15,
            max_len: 30,
            min_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(2),
            ..Default::default()
        });

        let noise = noise_gen.generate_noise_packet();
        assert!(noise.len() >= 15 && noise.len() <= 30);
    }
}
