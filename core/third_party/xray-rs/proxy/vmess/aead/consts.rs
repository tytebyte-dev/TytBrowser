// Module: proxy\vmess\aead\consts.rs
// 1:1 Rust implementation corresponding to Go proxy\vmess\aead\consts.go

pub const AUTH_ID_SIZE: usize = 16;
pub const VMESS_HEADER_TYPE: u8 = 1;
pub const KDF_SALT_CONST_VMESS_AEAD_KDF: &[u8] = b"VMess AEAD KDF";
pub const KDF_SALT_CONST_AUTH_ID_ENCRYPTION_KEY: &[u8] = b"AES Auth ID Encryption Key";
pub const KDF_SALT_CONST_AEAD_RESP_HEADER_LEN_KEY: &[u8] = b"AEAD Resp Header Len Key";
pub const KDF_SALT_CONST_AEAD_RESP_HEADER_LEN_IV: &[u8] = b"AEAD Resp Header Len IV";
pub const KDF_SALT_CONST_AEAD_RESP_HEADER_PAYLOAD_KEY: &[u8] = b"AEAD Resp Header Key";
pub const KDF_SALT_CONST_AEAD_RESP_HEADER_PAYLOAD_IV: &[u8] = b"AEAD Resp Header IV";
