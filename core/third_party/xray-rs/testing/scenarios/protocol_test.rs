#[cfg(test)]
mod tests {
    use crate::common::net::{Address, Destination};
    use crate::common::protocol::headers::{RequestCommand, RequestHeader};
    use crate::common::protocol::id::Id;
    use crate::common::protocol::user::{MemoryUser, SecurityType};
    use uuid::Uuid;

    #[test]
    fn test_protocol_id_cmd_key_derivation() {
        let raw_uuid = Uuid::parse_str("2ea73714-138e-4cc7-8cab-d7caf476d51b").unwrap();
        let id = Id::new(raw_uuid);
        assert_eq!(id.uuid(), raw_uuid);
        assert_eq!(id.bytes(), raw_uuid.as_bytes());
        // Verify deterministic MD5 cmd_key
        assert_eq!(id.cmd_key().len(), 16);
    }

    #[test]
    fn test_protocol_memory_user_and_request_header() {
        let raw_uuid = Uuid::new_v4();
        let user = MemoryUser::new(raw_uuid, "test@xray.com", 1);
        assert_eq!(user.email, "test@xray.com");
        assert_eq!(user.level, 1);

        let dest = Destination::new(Address::Domain("api.google.com".into()), 443);
        let mut req = RequestHeader::new(RequestCommand::Tcp, dest.clone());
        req.security = SecurityType::Aes128Gcm;
        req.user = Some(user);

        assert_eq!(req.command, RequestCommand::Tcp);
        assert_eq!(req.destination, dest);
        assert_eq!(req.security, SecurityType::Aes128Gcm);
        assert!(req.user.is_some());
    }
}
