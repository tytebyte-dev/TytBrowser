// Module: transport\internet\headers\noop\mod.rs
// 1:1 Rust implementation corresponding to Go transport\internet\headers\noop

#[path = "config.pb.rs"]
pub mod config_pb;
pub mod noop;

#[cfg(test)]
mod tests {
    use super::config_pb::{Config, ConnectionConfig};
    use super::noop::{NoOpConnectionHeader, NoOpHeader};
    use crate::transport::internet::header::{ConnectionAuthenticator, Header, PacketHeader};

    #[test]
    fn test_noop_headers() {
        let h = NoOpHeader::new();
        assert_eq!(Header::size(&h), 0);
        let mut buf = [0u8; 10];
        assert_eq!(Header::serialize(&h, &mut buf), 0);
        assert_eq!(PacketHeader::size(&h), 0);
        PacketHeader::serialize(&h, &mut buf);

        let ch = NoOpConnectionHeader::new();
        let (client, _server) = tokio::io::duplex(64);
        let boxed = ch.client(Box::pin(client));
        drop(boxed);

        let _cfg = Config::default();
        let _conn_cfg = ConnectionConfig::default();
    }
}

pub use config_pb::{Config, ConnectionConfig};
pub use noop::{NoOpConnectionHeader, NoOpHeader};
