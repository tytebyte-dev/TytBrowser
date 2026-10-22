// Module: proxy\vmess\encoding\encoding_test.rs
// 1:1 Rust unit test suite corresponding to Go proxy\vmess\encoding\encoding_test.go

#[cfg(test)]
mod tests {
    use super::super::VMESS_VERSION;
    use super::super::auth::{ShakeSizeParser, authenticate, generate_chacha20poly1305_key};
    use super::super::commands::{
        CMD_SWITCH_ACCOUNT, CMD_TCP, CMD_UDP, CommandSwitchAccount, marshal_command,
        unmarshal_command,
    };
    use uuid::Uuid;

    #[test]
    fn test_vmess_constants() {
        assert_eq!(CMD_TCP, 1);
        assert_eq!(CMD_UDP, 2);
        assert_eq!(VMESS_VERSION, 1);
    }

    #[test]
    fn test_vmess_auth_and_key() {
        let data = b"hello vmess auth";
        let h1 = authenticate(data);
        let h2 = authenticate(data);
        assert_eq!(h1, h2);
        assert_ne!(h1, 0);

        let seed = [1u8; 16];
        let key = generate_chacha20poly1305_key(&seed);
        assert_eq!(key.len(), 32);
        assert_ne!(&key[0..16], &key[16..32]);
    }

    #[test]
    fn test_shake_size_parser() {
        let nonce = [9u8; 16];
        let mut parser = ShakeSizeParser::new(&nonce);
        let mut parser2 = ShakeSizeParser::new(&nonce);

        let encoded = parser.encode(4096);
        let decoded = parser2.decode(&encoded).unwrap();
        assert_eq!(decoded, 4096);
    }

    #[test]
    fn test_switch_account_command_roundtrip() {
        let cmd = CommandSwitchAccount {
            host: "backup.vmess.com".into(),
            port: 10086,
            id: Uuid::new_v4(),
            alter_id: 16,
            level: 1,
            valid_min: 15,
        };

        let encoded = cmd.encode();
        let decoded = CommandSwitchAccount::decode(&encoded).unwrap();
        assert_eq!(cmd, decoded);

        let marshaled = marshal_command(CMD_SWITCH_ACCOUNT, &encoded).unwrap();
        let (out_id, out_payload) = unmarshal_command(&marshaled).unwrap();
        assert_eq!(out_id, CMD_SWITCH_ACCOUNT);
        assert_eq!(out_payload, encoded);
    }
}
