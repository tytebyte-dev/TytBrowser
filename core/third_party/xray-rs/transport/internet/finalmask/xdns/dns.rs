// Module: transport\internet\finalmask\xdns\dns.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\xdns\dns.go

use crate::common::errors::{Error, Result};
use std::collections::HashMap;
use std::fmt;

pub const COMPRESSION_POINTER_LIMIT: usize = 10;

pub const RR_TYPE_A: u16 = 1;
pub const RR_TYPE_NS: u16 = 2;
pub const RR_TYPE_CNAME: u16 = 5;
pub const RR_TYPE_SOA: u16 = 6;
pub const RR_TYPE_PTR: u16 = 12;
pub const RR_TYPE_MX: u16 = 15;
pub const RR_TYPE_TXT: u16 = 16;
pub const RR_TYPE_AAAA: u16 = 28;
pub const RR_TYPE_OPT: u16 = 41;

pub const CLASS_IN: u16 = 1;

pub const RCODE_NO_ERROR: u16 = 0;
pub const RCODE_FORMAT_ERROR: u16 = 1;
pub const RCODE_SERVER_FAILURE: u16 = 2;
pub const RCODE_NAME_ERROR: u16 = 3;
pub const RCODE_NOT_IMPLEMENTED: u16 = 4;
pub const RCODE_REFUSED: u16 = 5;
pub const EXTENDED_RCODE_BAD_VERS: u16 = 16;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Name {
    pub labels: Vec<Vec<u8>>,
}

impl Name {
    pub fn new(labels: Vec<Vec<u8>>) -> Result<Self> {
        let mut total_len = 1; // root null byte
        for label in &labels {
            if label.is_empty() {
                return Err(Error::Protocol("name contains a zero-length label".into()));
            }
            if label.len() > 63 {
                return Err(Error::Protocol(
                    "name contains a label longer than 63 octets".into(),
                ));
            }
            total_len += 1 + label.len();
        }
        if total_len > 255 {
            return Err(Error::Protocol("name is longer than 255 octets".into()));
        }
        Ok(Self { labels })
    }

    pub fn parse(s: &str) -> Result<Self> {
        let trimmed = s.trim_end_matches('.');
        if trimmed.is_empty() {
            return Ok(Self { labels: Vec::new() });
        }
        let labels: Vec<Vec<u8>> = trimmed.split('.').map(|p| p.as_bytes().to_vec()).collect();
        Self::new(labels)
    }

    pub fn trim_suffix(&self, suffix: &Name) -> Option<Name> {
        if self.labels.len() < suffix.labels.len() {
            return None;
        }
        let split_idx = self.labels.len() - suffix.labels.len();
        let (fore, aft) = self.labels.split_at(split_idx);
        for (a, b) in aft.iter().zip(suffix.labels.iter()) {
            if !a.eq_ignore_ascii_case(b) {
                return None;
            }
        }
        Some(Name {
            labels: fore.to_vec(),
        })
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.labels.is_empty() {
            return write!(f, ".");
        }
        for (i, label) in self.labels.iter().enumerate() {
            if i > 0 {
                write!(f, ".")?;
            }
            for &b in label {
                if b == b'-'
                    || b.is_ascii_digit()
                    || b.is_ascii_uppercase()
                    || b.is_ascii_lowercase()
                {
                    write!(f, "{}", b as char)?;
                } else {
                    write!(f, "\\x{:02x}", b)?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub name: Name,
    pub qtype: u16,
    pub qclass: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceRecord {
    pub name: Name,
    pub rrtype: u16,
    pub class: u16,
    pub ttl: u32,
    pub data: Vec<u8>,
}

pub type RR = ResourceRecord;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Message {
    pub id: u16,
    pub flags: u16,
    pub questions: Vec<Question>,
    pub answers: Vec<RR>,
    pub authority: Vec<RR>,
    pub additional: Vec<RR>,
}

impl Message {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn opcode(&self) -> u16 {
        (self.flags >> 11) & 0x0F
    }

    pub fn rcode(&self) -> u16 {
        self.flags & 0x0F
    }

    pub fn pack(&self) -> Result<Vec<u8>> {
        self.to_wire_format()
    }

    pub fn to_wire_format(&self) -> Result<Vec<u8>> {
        let mut builder = MessageBuilder::new();
        builder.write_message(self)?;
        Ok(builder.bytes().to_vec())
    }

    pub fn unpack(bytes: &[u8]) -> Result<Self> {
        Self::from_wire_format(bytes)
    }

    pub fn from_wire_format(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 12 {
            return Err(Error::Protocol("DNS message too short".into()));
        }

        let id = u16::from_be_bytes([bytes[0], bytes[1]]);
        let flags = u16::from_be_bytes([bytes[2], bytes[3]]);
        let qdcount = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
        let ancount = u16::from_be_bytes([bytes[6], bytes[7]]) as usize;
        let nscount = u16::from_be_bytes([bytes[8], bytes[9]]) as usize;
        let arcount = u16::from_be_bytes([bytes[10], bytes[11]]) as usize;

        let mut offset = 12;
        let mut questions = Vec::with_capacity(qdcount);
        for _ in 0..qdcount {
            let (name, next_off) = Self::read_name(bytes, offset)?;
            offset = next_off;
            if offset + 4 > bytes.len() {
                return Err(Error::Protocol("Truncated DNS question".into()));
            }
            let qtype = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
            let qclass = u16::from_be_bytes([bytes[offset + 2], bytes[offset + 3]]);
            offset += 4;
            questions.push(Question {
                name,
                qtype,
                qclass,
            });
        }

        let mut answers = Vec::with_capacity(ancount);
        for _ in 0..ancount {
            let (rr, next_off) = Self::read_rr(bytes, offset)?;
            offset = next_off;
            answers.push(rr);
        }

        let mut authority = Vec::with_capacity(nscount);
        for _ in 0..nscount {
            let (rr, next_off) = Self::read_rr(bytes, offset)?;
            offset = next_off;
            authority.push(rr);
        }

        let mut additional = Vec::with_capacity(arcount);
        for _ in 0..arcount {
            let (rr, next_off) = Self::read_rr(bytes, offset)?;
            offset = next_off;
            additional.push(rr);
        }

        if offset < bytes.len() {
            return Err(Error::Protocol("trailing bytes after message".into()));
        }

        Ok(Self {
            id,
            flags,
            questions,
            answers,
            authority,
            additional,
        })
    }

    fn read_name(bytes: &[u8], mut offset: usize) -> Result<(Name, usize)> {
        let mut labels = Vec::new();
        let mut jumps = 0;
        let mut return_offset = None;

        while offset < bytes.len() {
            let len_byte = bytes[offset];
            offset += 1;

            if len_byte == 0 {
                break;
            }

            match len_byte & 0xC0 {
                0x00 => {
                    let label_len = (len_byte & 0x3F) as usize;
                    if offset + label_len > bytes.len() {
                        return Err(Error::Protocol("Label truncated".into()));
                    }
                    labels.push(bytes[offset..offset + label_len].to_vec());
                    offset += label_len;
                }
                0xC0 => {
                    if offset >= bytes.len() {
                        return Err(Error::Protocol("Truncated compression pointer".into()));
                    }
                    let ptr_low = bytes[offset];
                    offset += 1;
                    let ptr = (((len_byte & 0x3F) as usize) << 8) | (ptr_low as usize);
                    if return_offset.is_none() {
                        return_offset = Some(offset);
                    }
                    offset = ptr;
                    jumps += 1;
                    if jumps > COMPRESSION_POINTER_LIMIT {
                        return Err(Error::Protocol("too many compression pointers".into()));
                    }
                }
                _ => return Err(Error::Protocol("reserved label type".into())),
            }
        }

        let final_offset = return_offset.unwrap_or(offset);
        let name = Name::new(labels)?;
        Ok((name, final_offset))
    }

    fn read_rr(bytes: &[u8], offset: usize) -> Result<(RR, usize)> {
        let (name, mut off) = Self::read_name(bytes, offset)?;
        if off + 10 > bytes.len() {
            return Err(Error::Protocol("Truncated RR header".into()));
        }
        let rrtype = u16::from_be_bytes([bytes[off], bytes[off + 1]]);
        let class = u16::from_be_bytes([bytes[off + 2], bytes[off + 3]]);
        let ttl = u32::from_be_bytes([
            bytes[off + 4],
            bytes[off + 5],
            bytes[off + 6],
            bytes[off + 7],
        ]);
        let rdlength = u16::from_be_bytes([bytes[off + 8], bytes[off + 9]]) as usize;
        off += 10;

        if off + rdlength > bytes.len() {
            return Err(Error::Protocol("Truncated RDATA".into()));
        }
        let data = bytes[off..off + rdlength].to_vec();
        off += rdlength;

        Ok((
            ResourceRecord {
                name,
                rrtype,
                class,
                ttl,
                data,
            },
            off,
        ))
    }
}

pub struct MessageBuilder {
    buf: Vec<u8>,
    name_cache: HashMap<String, usize>,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            name_cache: HashMap::new(),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.buf
    }

    pub fn write_name(&mut self, name: &Name) {
        for i in 0..name.labels.len() {
            let suffix_name = Name {
                labels: name.labels[i..].to_vec(),
            };
            let key = suffix_name.to_string();
            if let Some(&ptr) = self.name_cache.get(&key)
                && (ptr & 0x3fff) == ptr
            {
                let comp = 0xc000 | (ptr as u16);
                self.buf.extend_from_slice(&comp.to_be_bytes());
                return;
            }
            self.name_cache.insert(key, self.buf.len());
            let label = &name.labels[i];
            self.buf.push(label.len() as u8);
            self.buf.extend_from_slice(label);
        }
        self.buf.push(0);
    }

    pub fn write_question(&mut self, q: &Question) {
        self.write_name(&q.name);
        self.buf.extend_from_slice(&q.qtype.to_be_bytes());
        self.buf.extend_from_slice(&q.qclass.to_be_bytes());
    }

    pub fn write_rr(&mut self, rr: &RR) -> Result<()> {
        self.write_name(&rr.name);
        self.buf.extend_from_slice(&rr.rrtype.to_be_bytes());
        self.buf.extend_from_slice(&rr.class.to_be_bytes());
        self.buf.extend_from_slice(&rr.ttl.to_be_bytes());
        if rr.data.len() > 65535 {
            return Err(Error::Protocol("integer overflow".into()));
        }
        self.buf
            .extend_from_slice(&(rr.data.len() as u16).to_be_bytes());
        self.buf.extend_from_slice(&rr.data);
        Ok(())
    }

    pub fn write_message(&mut self, msg: &Message) -> Result<()> {
        self.buf.extend_from_slice(&msg.id.to_be_bytes());
        self.buf.extend_from_slice(&msg.flags.to_be_bytes());
        for count in [
            msg.questions.len(),
            msg.answers.len(),
            msg.authority.len(),
            msg.additional.len(),
        ] {
            if count > 65535 {
                return Err(Error::Protocol("integer overflow".into()));
            }
            self.buf.extend_from_slice(&(count as u16).to_be_bytes());
        }
        for q in &msg.questions {
            self.write_question(q);
        }
        for rr in &msg.answers {
            self.write_rr(rr)?;
        }
        for rr in &msg.authority {
            self.write_rr(rr)?;
        }
        for rr in &msg.additional {
            self.write_rr(rr)?;
        }
        Ok(())
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Encodes a slice of bytes as TXT-DATA (RFC 1035 section 3.3.14).
pub fn encode_rdata_txt(p: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut remaining = p;
    while remaining.len() > 255 {
        buf.push(255);
        buf.extend_from_slice(&remaining[..255]);
        remaining = &remaining[255..];
    }
    buf.push(remaining.len() as u8);
    buf.extend_from_slice(remaining);
    buf
}

/// Decodes TXT-DATA (RFC 1035 section 3.3.14).
pub fn decode_rdata_txt(p: &[u8]) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut offset = 0;
    if p.is_empty() {
        return Err(Error::Protocol("unexpected eof in TXT RDATA".into()));
    }
    while offset < p.len() {
        let n = p[offset] as usize;
        offset += 1;
        if offset + n > p.len() {
            return Err(Error::Protocol("unexpected eof in TXT RDATA".into()));
        }
        buf.extend_from_slice(&p[offset..offset + n]);
        offset += n;
    }
    Ok(buf)
}

/// Standard Base32 (RFC 4648) encoding without padding.
pub fn base32_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut out = String::new();
    let mut buffer = 0u64;
    let mut bits_left = 0;
    for &b in data {
        buffer = (buffer << 8) | (b as u64);
        bits_left += 8;
        while bits_left >= 5 {
            bits_left -= 5;
            let index = ((buffer >> bits_left) & 0x1f) as usize;
            out.push(ALPHABET[index] as char);
        }
    }
    if bits_left > 0 {
        let index = ((buffer << (5 - bits_left)) & 0x1f) as usize;
        out.push(ALPHABET[index] as char);
    }
    out
}

/// Standard Base32 (RFC 4648) decoding without padding, case-insensitive.
pub fn base32_decode(s: &str) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut buffer = 0u64;
    let mut bits_left = 0;
    for c in s.chars() {
        if c.is_whitespace() {
            continue;
        }
        let val = match c {
            'A'..='Z' => (c as u8 - b'A') as u64,
            'a'..='z' => (c as u8 - b'a') as u64,
            '2'..='7' => (c as u8 - b'2' + 26) as u64,
            _ => return Err(Error::Protocol(format!("invalid base32 char: {}", c))),
        };
        buffer = (buffer << 5) | val;
        bits_left += 5;
        if bits_left >= 8 {
            bits_left -= 8;
            out.push(((buffer >> bits_left) & 0xff) as u8);
        }
    }
    Ok(out)
}
