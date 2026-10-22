// Module: transport\internet\kcp\segment.rs
// 1:1 Rust implementation corresponding to Go transport\internet\kcp\segment.go

use crate::common::errors::{Error, Result};

pub const COMMAND_ACK: u8 = 0;
pub const COMMAND_DATA: u8 = 1;
pub const COMMAND_TERMINATE: u8 = 2;
pub const COMMAND_PING: u8 = 3;

pub const OPTION_CLOSE: u8 = 1;

pub const DATA_SEGMENT_OVERHEAD: usize = 18;
pub const ACK_NUMBER_LIMIT: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    Data(DataSegment),
    Ack(AckSegment),
    CmdOnly(CmdOnlySegment),
}

impl Segment {
    pub fn conversation(&self) -> u16 {
        match self {
            Segment::Data(s) => s.conv,
            Segment::Ack(s) => s.conv,
            Segment::CmdOnly(s) => s.conv,
        }
    }

    pub fn command(&self) -> u8 {
        match self {
            Segment::Data(_) => COMMAND_DATA,
            Segment::Ack(_) => COMMAND_ACK,
            Segment::CmdOnly(s) => s.cmd,
        }
    }

    pub fn byte_size(&self) -> usize {
        match self {
            Segment::Data(s) => s.byte_size(),
            Segment::Ack(s) => s.byte_size(),
            Segment::CmdOnly(s) => s.byte_size(),
        }
    }

    pub fn serialize(&self, buf: &mut [u8]) -> Result<usize> {
        match self {
            Segment::Data(s) => s.serialize(buf),
            Segment::Ack(s) => s.serialize(buf),
            Segment::CmdOnly(s) => s.serialize(buf),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSegment {
    pub conv: u16,
    pub option: u8,
    pub timestamp: u32,
    pub number: u32,
    pub sending_next: u32,
    pub payload: Vec<u8>,
    pub timeout: u32,
    pub transmit: u32,
}

impl DataSegment {
    pub fn new(conv: u16, number: u32, payload: Vec<u8>) -> Self {
        Self {
            conv,
            option: 0,
            timestamp: 0,
            number,
            sending_next: number + 1,
            payload,
            timeout: 0,
            transmit: 0,
        }
    }

    pub fn byte_size(&self) -> usize {
        DATA_SEGMENT_OVERHEAD + self.payload.len()
    }

    pub fn serialize(&self, buf: &mut [u8]) -> Result<usize> {
        let total = self.byte_size();
        if buf.len() < total {
            return Err(Error::BufferOverflow);
        }
        buf[0..2].copy_from_slice(&self.conv.to_be_bytes());
        buf[2] = COMMAND_DATA;
        buf[3] = self.option;
        buf[4..8].copy_from_slice(&self.timestamp.to_be_bytes());
        buf[8..12].copy_from_slice(&self.number.to_be_bytes());
        buf[12..16].copy_from_slice(&self.sending_next.to_be_bytes());
        buf[16..18].copy_from_slice(&(self.payload.len() as u16).to_be_bytes());
        buf[18..total].copy_from_slice(&self.payload);
        Ok(total)
    }

    pub fn decode(buf: &[u8]) -> Result<Self> {
        let (seg, _) = read_segment(buf)
            .ok_or_else(|| Error::Protocol("failed to parse data segment".into()))?;
        match seg {
            Segment::Data(d) => Ok(d),
            _ => Err(Error::Protocol("expected data segment".into())),
        }
    }

    pub fn parse(conv: u16, option: u8, buf: &[u8]) -> Option<(Self, &[u8])> {
        if buf.len() < 14 {
            return None;
        }
        let timestamp = u32::from_be_bytes(buf[0..4].try_into().unwrap());
        let number = u32::from_be_bytes(buf[4..8].try_into().unwrap());
        let sending_next = u32::from_be_bytes(buf[8..12].try_into().unwrap());
        let data_len = u16::from_be_bytes(buf[12..14].try_into().unwrap()) as usize;
        let rest = &buf[14..];
        if rest.len() < data_len {
            return None;
        }
        let payload = rest[..data_len].to_vec();
        let remaining = &rest[data_len..];

        Some((
            Self {
                conv,
                option,
                timestamp,
                number,
                sending_next,
                payload,
                timeout: 0,
                transmit: 0,
            },
            remaining,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AckSegment {
    pub conv: u16,
    pub option: u8,
    pub receiving_window: u32,
    pub receiving_next: u32,
    pub timestamp: u32,
    pub number_list: Vec<u32>,
    pub limit: usize,
}

impl AckSegment {
    pub fn new(conv: u16, limit: usize) -> Self {
        let lim = limit.clamp(1, ACK_NUMBER_LIMIT);
        Self {
            conv,
            option: 0,
            receiving_window: 0,
            receiving_next: 0,
            timestamp: 0,
            number_list: Vec::with_capacity(lim),
            limit: lim,
        }
    }

    pub fn put_timestamp(&mut self, timestamp: u32) {
        if timestamp.wrapping_sub(self.timestamp) < 0x7FFFFFFF {
            self.timestamp = timestamp;
        }
    }

    pub fn put_number(&mut self, number: u32) {
        if self.number_list.len() < self.limit {
            self.number_list.push(number);
        }
    }

    pub fn is_full(&self) -> bool {
        self.number_list.len() >= self.limit
    }

    pub fn is_empty(&self) -> bool {
        self.number_list.is_empty()
    }

    pub fn byte_size(&self) -> usize {
        17 + self.number_list.len() * 4
    }

    pub fn serialize(&self, buf: &mut [u8]) -> Result<usize> {
        let total = self.byte_size();
        if buf.len() < total {
            return Err(Error::BufferOverflow);
        }
        buf[0..2].copy_from_slice(&self.conv.to_be_bytes());
        buf[2] = COMMAND_ACK;
        buf[3] = self.option;
        buf[4..8].copy_from_slice(&self.receiving_window.to_be_bytes());
        buf[8..12].copy_from_slice(&self.receiving_next.to_be_bytes());
        buf[12..16].copy_from_slice(&self.timestamp.to_be_bytes());
        buf[16] = self.number_list.len() as u8;

        let mut offset = 17;
        for &num in &self.number_list {
            buf[offset..offset + 4].copy_from_slice(&num.to_be_bytes());
            offset += 4;
        }
        Ok(total)
    }

    pub fn decode(buf: &[u8]) -> Result<Self> {
        let (seg, _) = read_segment(buf)
            .ok_or_else(|| Error::Protocol("failed to parse ack segment".into()))?;
        match seg {
            Segment::Ack(a) => Ok(a),
            _ => Err(Error::Protocol("expected ack segment".into())),
        }
    }

    pub fn parse(conv: u16, option: u8, buf: &[u8]) -> Option<(Self, &[u8])> {
        if buf.len() < 13 {
            return None;
        }
        let receiving_window = u32::from_be_bytes(buf[0..4].try_into().unwrap());
        let receiving_next = u32::from_be_bytes(buf[4..8].try_into().unwrap());
        let timestamp = u32::from_be_bytes(buf[8..12].try_into().unwrap());
        let count = buf[12] as usize;
        let mut rest = &buf[13..];
        if rest.len() < count * 4 {
            return None;
        }
        let mut number_list = Vec::with_capacity(count);
        for _ in 0..count {
            let num = u32::from_be_bytes(rest[0..4].try_into().unwrap());
            number_list.push(num);
            rest = &rest[4..];
        }

        Some((
            Self {
                conv,
                option,
                receiving_window,
                receiving_next,
                timestamp,
                number_list,
                limit: ACK_NUMBER_LIMIT,
            },
            rest,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmdOnlySegment {
    pub conv: u16,
    pub cmd: u8,
    pub option: u8,
    pub sending_next: u32,
    pub receiving_next: u32,
    pub peer_rto: u32,
}

impl CmdOnlySegment {
    pub fn new(conv: u16, cmd: u8) -> Self {
        Self {
            conv,
            cmd,
            option: 0,
            sending_next: 0,
            receiving_next: 0,
            peer_rto: 0,
        }
    }

    pub fn byte_size(&self) -> usize {
        16
    }

    pub fn serialize(&self, buf: &mut [u8]) -> Result<usize> {
        if buf.len() < 16 {
            return Err(Error::BufferOverflow);
        }
        buf[0..2].copy_from_slice(&self.conv.to_be_bytes());
        buf[2] = self.cmd;
        buf[3] = self.option;
        buf[4..8].copy_from_slice(&self.sending_next.to_be_bytes());
        buf[8..12].copy_from_slice(&self.receiving_next.to_be_bytes());
        buf[12..16].copy_from_slice(&self.peer_rto.to_be_bytes());
        Ok(16)
    }

    pub fn parse(conv: u16, cmd: u8, option: u8, buf: &[u8]) -> Option<(Self, &[u8])> {
        if buf.len() < 12 {
            return None;
        }
        let sending_next = u32::from_be_bytes(buf[0..4].try_into().unwrap());
        let receiving_next = u32::from_be_bytes(buf[4..8].try_into().unwrap());
        let peer_rto = u32::from_be_bytes(buf[8..12].try_into().unwrap());
        Some((
            Self {
                conv,
                cmd,
                option,
                sending_next,
                receiving_next,
                peer_rto,
            },
            &buf[12..],
        ))
    }
}

pub fn read_segment(buf: &[u8]) -> Option<(Segment, &[u8])> {
    if buf.len() < 4 {
        return None;
    }
    let conv = u16::from_be_bytes([buf[0], buf[1]]);
    let cmd = buf[2];
    let opt = buf[3];
    let rest = &buf[4..];

    match cmd {
        COMMAND_DATA => {
            let (seg, remaining) = DataSegment::parse(conv, opt, rest)?;
            Some((Segment::Data(seg), remaining))
        }
        COMMAND_ACK => {
            let (seg, remaining) = AckSegment::parse(conv, opt, rest)?;
            Some((Segment::Ack(seg), remaining))
        }
        _ => {
            let (seg, remaining) = CmdOnlySegment::parse(conv, cmd, opt, rest)?;
            Some((Segment::CmdOnly(seg), remaining))
        }
    }
}
