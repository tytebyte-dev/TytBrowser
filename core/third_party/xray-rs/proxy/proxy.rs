// Module: proxy\proxy.rs
// 1:1 Rust implementation corresponding to Go proxy\proxy.go
// Includes core proxy traits and the complete XTLS-Vision state machine & padding engine.

use crate::common::net::Network;
use rand::Rng;
use std::collections::HashMap;

pub const PROXY_VERSION: &str = "26.3.27";

pub const TLS13_SUPPORTED_VERSIONS: &[u8] = &[0x00, 0x2b, 0x00, 0x02, 0x03, 0x04];
pub const TLS_CLIENT_HANDSHAKE_START: &[u8] = &[0x16, 0x03];
pub const TLS_SERVER_HANDSHAKE_START: &[u8] = &[0x16, 0x03, 0x03];
pub const TLS_APPLICATION_DATA_START: &[u8] = &[0x17, 0x03, 0x03];

pub const TLS_HANDSHAKE_TYPE_CLIENT_HELLO: u8 = 0x01;
pub const TLS_HANDSHAKE_TYPE_SERVER_HELLO: u8 = 0x02;

pub const COMMAND_PADDING_CONTINUE: u8 = 0x00;
pub const COMMAND_PADDING_END: u8 = 0x01;
pub const COMMAND_PADDING_DIRECT: u8 = 0x02;

pub const BUFFER_SIZE: usize = 2048;

lazy_static::lazy_static! {
    pub static ref TLS13_CIPHER_SUITE_DIC: HashMap<u16, &'static str> = {
        let mut m = HashMap::new();
        m.insert(0x1301, "TLS_AES_128_GCM_SHA256");
        m.insert(0x1302, "TLS_AES_256_GCM_SHA384");
        m.insert(0x1303, "TLS_CHACHA20_POLY1305_SHA256");
        m.insert(0x1304, "TLS_AES_128_CCM_SHA256");
        m.insert(0x1305, "TLS_AES_128_CCM_8_SHA256");
        m
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboundState {
    pub within_padding_buffers: bool,
    pub uplink_reader_direct_copy: bool,
    pub remaining_command: i32,
    pub remaining_content: i32,
    pub remaining_padding: i32,
    pub current_command: i32,
    pub is_padding: bool,
    pub downlink_writer_direct_copy: bool,
}

impl Default for InboundState {
    fn default() -> Self {
        Self {
            within_padding_buffers: true,
            uplink_reader_direct_copy: false,
            remaining_command: -1,
            remaining_content: -1,
            remaining_padding: -1,
            current_command: 0,
            is_padding: true,
            downlink_writer_direct_copy: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundState {
    pub within_padding_buffers: bool,
    pub downlink_reader_direct_copy: bool,
    pub remaining_command: i32,
    pub remaining_content: i32,
    pub remaining_padding: i32,
    pub current_command: i32,
    pub is_padding: bool,
    pub uplink_writer_direct_copy: bool,
}

impl Default for OutboundState {
    fn default() -> Self {
        Self {
            within_padding_buffers: true,
            downlink_reader_direct_copy: false,
            remaining_command: -1,
            remaining_content: -1,
            remaining_padding: -1,
            current_command: 0,
            is_padding: true,
            uplink_writer_direct_copy: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrafficState {
    pub user_uuid: Vec<u8>,
    pub number_of_packet_to_filter: i32,
    pub enable_xtls: bool,
    pub is_tls12_or_above: bool,
    pub is_tls: bool,
    pub cipher: u16,
    pub remaining_server_hello: i32,
    pub inbound: InboundState,
    pub outbound: OutboundState,
}

impl TrafficState {
    pub fn new(user_uuid: &[u8]) -> Self {
        Self {
            user_uuid: user_uuid.to_vec(),
            number_of_packet_to_filter: 8,
            enable_xtls: false,
            is_tls12_or_above: false,
            is_tls: false,
            cipher: 0,
            remaining_server_hello: -1,
            inbound: InboundState::default(),
            outbound: OutboundState::default(),
        }
    }
}

pub fn is_complete_record(data: &[u8]) -> bool {
    let mut header_len = 5;
    let mut record_len = 0usize;
    let total_len = data.len();
    let mut i = 0;

    while i < total_len {
        if header_len > 0 {
            let b = data[i];
            i += 1;
            match header_len {
                5 => {
                    if b != 0x17 {
                        return false;
                    }
                }
                4 | 3 => {
                    if b != 0x03 {
                        return false;
                    }
                }
                2 => record_len = (b as usize) << 8,
                1 => record_len |= b as usize,
                _ => return false,
            }
            header_len -= 1;
        } else if record_len > 0 {
            let remaining = total_len - i;
            if remaining < record_len {
                return false;
            } else {
                i += record_len;
                record_len = 0;
                header_len = 5;
            }
        } else {
            return false;
        }
    }

    header_len == 5 && record_len == 0
}

pub fn xtls_padding(
    content: Option<&[u8]>,
    command: u8,
    user_uuid: Option<&[u8]>,
    long_padding: bool,
    testseed: Option<&[u32]>,
) -> Vec<u8> {
    let seed = testseed.unwrap_or(&[900, 500, 900, 256]);
    let content_len = content.map(|c| c.len()).unwrap_or(0) as i32;

    let mut rng = rand::thread_rng();
    let mut padding_len = if (content_len as u32) < seed[0] && long_padding {
        let rand_val = if seed[1] > 0 {
            rng.gen_range(0..seed[1]) as i32
        } else {
            0
        };
        rand_val + (seed[2] as i32) - content_len
    } else {
        if seed[3] > 0 {
            rng.gen_range(0..seed[3]) as i32
        } else {
            0
        }
    };

    let max_padding = (BUFFER_SIZE as i32) - 21 - content_len;
    if padding_len > max_padding {
        padding_len = max_padding.max(0);
    }
    if padding_len < 0 {
        padding_len = 0;
    }

    let mut out = Vec::new();
    if let Some(uuid) = user_uuid {
        out.extend_from_slice(uuid);
    }

    out.push(command);
    out.push((content_len >> 8) as u8);
    out.push(content_len as u8);
    out.push((padding_len >> 8) as u8);
    out.push(padding_len as u8);

    if let Some(c) = content {
        out.extend_from_slice(c);
    }

    if padding_len > 0 {
        out.resize(out.len() + padding_len as usize, 0u8);
    }

    out
}

pub fn xtls_unpadding(data: &[u8], state: &mut TrafficState, is_uplink: bool) -> Vec<u8> {
    let (remaining_command, remaining_content, remaining_padding, current_command) = if is_uplink {
        (
            &mut state.inbound.remaining_command,
            &mut state.inbound.remaining_content,
            &mut state.inbound.remaining_padding,
            &mut state.inbound.current_command,
        )
    } else {
        (
            &mut state.outbound.remaining_command,
            &mut state.outbound.remaining_content,
            &mut state.outbound.remaining_padding,
            &mut state.outbound.current_command,
        )
    };

    let mut cursor = 0;
    let len = data.len();

    if *remaining_command == -1 && *remaining_content == -1 && *remaining_padding == -1 {
        if len >= 21 && &data[..16] == state.user_uuid.as_slice() {
            cursor += 16;
            *remaining_command = 5;
        } else {
            return data.to_vec();
        }
    }

    let mut result = Vec::new();

    while cursor < len {
        if *remaining_command > 0 {
            let byte = data[cursor];
            cursor += 1;
            match *remaining_command {
                5 => *current_command = byte as i32,
                4 => *remaining_content = (byte as i32) << 8,
                3 => *remaining_content |= byte as i32,
                2 => *remaining_padding = (byte as i32) << 8,
                1 => *remaining_padding |= byte as i32,
                _ => {}
            }
            *remaining_command -= 1;
        } else if *remaining_content > 0 {
            let to_read = (*remaining_content as usize).min(len - cursor);
            result.extend_from_slice(&data[cursor..cursor + to_read]);
            cursor += to_read;
            *remaining_content -= to_read as i32;
        } else if *remaining_padding > 0 {
            let to_skip = (*remaining_padding as usize).min(len - cursor);
            cursor += to_skip;
            *remaining_padding -= to_skip as i32;
        }

        if *remaining_command <= 0 && *remaining_content <= 0 && *remaining_padding <= 0 {
            if *current_command == COMMAND_PADDING_CONTINUE as i32 {
                *remaining_command = 5;
            } else {
                *remaining_command = -1;
                *remaining_content = -1;
                *remaining_padding = -1;
                if cursor < len {
                    result.extend_from_slice(&data[cursor..]);
                }
                break;
            }
        }
    }

    result
}

pub fn xtls_filter_tls(data: &[u8], traffic_state: &mut TrafficState) {
    if data.is_empty() {
        return;
    }

    traffic_state.number_of_packet_to_filter -= 1;

    if data.len() >= 6 {
        if data.len() >= 3
            && &data[..3] == TLS_SERVER_HANDSHAKE_START
            && data[5] == TLS_HANDSHAKE_TYPE_SERVER_HELLO
        {
            traffic_state.remaining_server_hello = ((data[3] as i32) << 8 | (data[4] as i32)) + 5;
            traffic_state.is_tls12_or_above = true;
            traffic_state.is_tls = true;
            if data.len() >= 79 && traffic_state.remaining_server_hello >= 79 {
                let session_id_len = data[43] as usize;
                if data.len() >= 43 + session_id_len + 3 {
                    traffic_state.cipher = (data[43 + session_id_len + 1] as u16) << 8
                        | (data[43 + session_id_len + 2] as u16);
                }
            }
        } else if data.len() >= 2
            && &data[..2] == TLS_CLIENT_HANDSHAKE_START
            && data[5] == TLS_HANDSHAKE_TYPE_CLIENT_HELLO
        {
            traffic_state.is_tls = true;
        }
    }

    if traffic_state.remaining_server_hello > 0 {
        let end = (traffic_state.remaining_server_hello as usize).min(data.len());
        traffic_state.remaining_server_hello -= data.len() as i32;

        if data[..end]
            .windows(TLS13_SUPPORTED_VERSIONS.len())
            .any(|w| w == TLS13_SUPPORTED_VERSIONS)
        {
            if let Some(v) = TLS13_CIPHER_SUITE_DIC.get(&traffic_state.cipher)
                && *v != "TLS_AES_128_CCM_8_SHA256"
            {
                traffic_state.enable_xtls = true;
            }
            traffic_state.number_of_packet_to_filter = 0;
        } else if traffic_state.remaining_server_hello <= 0 {
            traffic_state.number_of_packet_to_filter = 0;
        }
    }
}

pub trait Inbound: Send + Sync {
    fn network(&self) -> Vec<Network>;
}

pub trait Outbound: Send + Sync {
    fn tag(&self) -> &str;
}

pub trait UserManager: Send + Sync {
    fn add_user(&self, email: &str, level: u32) -> crate::common::errors::Result<()>;
    fn remove_user(&self, email: &str) -> crate::common::errors::Result<()>;
    fn get_users_count(&self) -> usize;
}

#[derive(Default)]
pub struct DefaultUserManager {
    users: std::sync::RwLock<HashMap<String, u32>>,
}

impl DefaultUserManager {
    pub fn new() -> Self {
        Self {
            users: std::sync::RwLock::new(HashMap::new()),
        }
    }
}

impl UserManager for DefaultUserManager {
    fn add_user(&self, email: &str, level: u32) -> crate::common::errors::Result<()> {
        let mut map = self.users.write().unwrap();
        map.insert(email.to_string(), level);
        Ok(())
    }

    fn remove_user(&self, email: &str) -> crate::common::errors::Result<()> {
        let mut map = self.users.write().unwrap();
        map.remove(email);
        Ok(())
    }

    fn get_users_count(&self) -> usize {
        self.users.read().map(|m| m.len()).unwrap_or(0)
    }
}
