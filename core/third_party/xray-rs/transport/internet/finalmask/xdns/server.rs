// Module: transport\internet\finalmask\xdns\server.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\xdns\server.go

use super::dns::{
    EXTENDED_RCODE_BAD_VERS, Message, Name, RCODE_FORMAT_ERROR, RCODE_NAME_ERROR, RCODE_NO_ERROR,
    RCODE_NOT_IMPLEMENTED, RR_TYPE_OPT, RR_TYPE_TXT, ResourceRecord, base32_decode,
    encode_rdata_txt,
};
use crate::common::errors::{Error, Result};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub const RESPONSE_TTL: u32 = 60;
pub const MAX_UDP_PAYLOAD: usize = 1280 - 40 - 8; // 1232 bytes (IPv6 MTU 1280 - 40 byte IPv6 header - 8 byte UDP header)

/// Represents a query decoded by the XDns server.
#[derive(Debug, Clone)]
pub struct XDnsServerQuery {
    pub client_id: [u8; 8],
    pub packets: Vec<Vec<u8>>,
    pub rcode: u16,
}

#[derive(Clone)]
pub struct XDnsServer {
    pub domain: Name,
    closed: Arc<AtomicBool>,
}

impl XDnsServer {
    pub fn new(domain_str: &str) -> Result<Self> {
        let domain = Name::parse(domain_str)?;
        Ok(Self {
            domain,
            closed: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn from_config(config: &super::config::XDnsConfig) -> Result<Self> {
        Self::new(&config.domain)
    }

    /// Converts an 8-byte client ID to a standardized synthetic IPv6 address for session tracking.
    pub fn client_id_to_addr(client_id: &[u8; 8]) -> [u8; 16] {
        let mut ip = [0u8; 16];
        ip[0] = 0xfd;
        ip[8..16].copy_from_slice(client_id);
        ip
    }

    /// Decodes an incoming DNS query wire format.
    /// Returns the parsed client query details along with the response Message skeleton.
    pub fn decode_query(&self, wire: &[u8]) -> Result<(XDnsServerQuery, Message)> {
        let query = Message::from_wire_format(wire)?;

        if (query.flags & 0x8000) != 0 {
            return Err(Error::Protocol(
                "incoming message is a DNS response, not a query".into(),
            ));
        }

        let mut resp = Message {
            id: query.id,
            flags: 0x8000, // Standard response
            questions: query.questions.clone(),
            answers: Vec::new(),
            authority: Vec::new(),
            additional: Vec::new(),
        };

        // Check for EDNS0 OPT resource record in additional section
        let mut payload_size = 512usize;
        for rr in &query.additional {
            if rr.rrtype != RR_TYPE_OPT {
                continue;
            }
            if !resp.additional.is_empty() {
                resp.flags |= RCODE_FORMAT_ERROR;
                return Ok((
                    XDnsServerQuery {
                        client_id: [0u8; 8],
                        packets: Vec::new(),
                        rcode: RCODE_FORMAT_ERROR,
                    },
                    resp,
                ));
            }

            let mut opt_rr = ResourceRecord {
                name: Name::default(),
                rrtype: RR_TYPE_OPT,
                class: 4096,
                ttl: 0,
                data: Vec::new(),
            };

            let version = ((rr.ttl >> 16) & 0xff) as u16;
            if version != 0 {
                resp.flags |= EXTENDED_RCODE_BAD_VERS & 0x0f;
                opt_rr.ttl = ((EXTENDED_RCODE_BAD_VERS as u32) >> 4) << 24;
                resp.additional.push(opt_rr);
                return Ok((
                    XDnsServerQuery {
                        client_id: [0u8; 8],
                        packets: Vec::new(),
                        rcode: EXTENDED_RCODE_BAD_VERS & 0x0f,
                    },
                    resp,
                ));
            }

            payload_size = std::cmp::max(512, rr.class as usize);
            resp.additional.push(opt_rr);
        }

        if query.questions.len() != 1 {
            resp.flags |= RCODE_FORMAT_ERROR;
            return Ok((
                XDnsServerQuery {
                    client_id: [0u8; 8],
                    packets: Vec::new(),
                    rcode: RCODE_FORMAT_ERROR,
                },
                resp,
            ));
        }

        let question = &query.questions[0];
        let prefix = match question.name.trim_suffix(&self.domain) {
            Some(p) => p,
            None => {
                resp.flags |= RCODE_NAME_ERROR;
                return Ok((
                    XDnsServerQuery {
                        client_id: [0u8; 8],
                        packets: Vec::new(),
                        rcode: RCODE_NAME_ERROR,
                    },
                    resp,
                ));
            }
        };

        resp.flags |= 0x0400; // Authoritative Answer (AA)

        if query.opcode() != 0 {
            resp.flags |= RCODE_NOT_IMPLEMENTED;
            return Ok((
                XDnsServerQuery {
                    client_id: [0u8; 8],
                    packets: Vec::new(),
                    rcode: RCODE_NOT_IMPLEMENTED,
                },
                resp,
            ));
        }

        if question.qtype != RR_TYPE_TXT {
            resp.flags |= RCODE_NAME_ERROR;
            return Ok((
                XDnsServerQuery {
                    client_id: [0u8; 8],
                    packets: Vec::new(),
                    rcode: RCODE_NAME_ERROR,
                },
                resp,
            ));
        }

        let mut concat = String::new();
        for label in &prefix.labels {
            concat.push_str(&String::from_utf8_lossy(label).to_ascii_uppercase());
        }

        let payload = match base32_decode(&concat) {
            Ok(p) => p,
            Err(_) => {
                resp.flags |= RCODE_NAME_ERROR;
                return Ok((
                    XDnsServerQuery {
                        client_id: [0u8; 8],
                        packets: Vec::new(),
                        rcode: RCODE_NAME_ERROR,
                    },
                    resp,
                ));
            }
        };

        if payload.len() < 8 {
            resp.flags |= RCODE_NAME_ERROR;
            return Ok((
                XDnsServerQuery {
                    client_id: [0u8; 8],
                    packets: Vec::new(),
                    rcode: RCODE_NAME_ERROR,
                },
                resp,
            ));
        }

        let mut client_id = [0u8; 8];
        client_id.copy_from_slice(&payload[..8]);

        let mut packets = Vec::new();
        let mut offset = 8;
        while offset < payload.len() {
            let prefix_byte = payload[offset];
            offset += 1;
            if prefix_byte >= 224 {
                let padding_len = (prefix_byte - 224) as usize;
                offset += padding_len;
            } else {
                let pkt_len = prefix_byte as usize;
                if offset + pkt_len > payload.len() {
                    break;
                }
                packets.push(payload[offset..offset + pkt_len].to_vec());
                offset += pkt_len;
            }
        }

        if payload_size < MAX_UDP_PAYLOAD {
            resp.flags |= RCODE_FORMAT_ERROR;
            return Ok((
                XDnsServerQuery {
                    client_id,
                    packets,
                    rcode: RCODE_FORMAT_ERROR,
                },
                resp,
            ));
        }

        Ok((
            XDnsServerQuery {
                client_id,
                packets,
                rcode: RCODE_NO_ERROR,
            },
            resp,
        ))
    }

    /// Encodes a response Message with outgoing packets into a wire-format DNS response.
    pub fn encode_response(
        &self,
        resp: &mut Message,
        outgoing_packets: &[&[u8]],
    ) -> Result<Vec<u8>> {
        if resp.rcode() == RCODE_NO_ERROR && resp.questions.len() == 1 {
            let mut payload = Vec::new();
            for pkt in outgoing_packets {
                if pkt.len() > 65535 {
                    continue;
                }
                payload.extend_from_slice(&(pkt.len() as u16).to_be_bytes());
                payload.extend_from_slice(pkt);
            }

            resp.answers = vec![ResourceRecord {
                name: resp.questions[0].name.clone(),
                rrtype: resp.questions[0].qtype,
                class: resp.questions[0].qclass,
                ttl: RESPONSE_TTL,
                data: encode_rdata_txt(&payload),
            }];
        }

        let mut wire = resp.to_wire_format()?;
        if wire.len() > MAX_UDP_PAYLOAD {
            wire.truncate(MAX_UDP_PAYLOAD);
            wire[2] |= 0x02; // Set TC (Truncated) bit
        }

        Ok(wire)
    }

    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }

    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }
}
