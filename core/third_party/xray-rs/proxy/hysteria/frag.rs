// Module: proxy\hysteria\frag.rs
// 1:1 Rust implementation corresponding to Go proxy\hysteria\frag.go

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UDPMessage {
    pub session_id: u32,
    pub packet_id: u16,
    pub frag_id: u8,
    pub frag_count: u8,
    pub address: String,
    pub data: Vec<u8>,
}

impl UDPMessage {
    pub fn new(session_id: u32, packet_id: u16, address: String, data: Vec<u8>) -> Self {
        Self {
            session_id,
            packet_id,
            frag_id: 0,
            frag_count: 1,
            address,
            data,
        }
    }

    pub fn header_size(&self) -> usize {
        // approximate UDP message header overhead: session(4) + packet_id(2) + frag_id(1) + frag_count(1) + addr_len(2) + addr
        8 + 2 + self.address.len()
    }

    pub fn size(&self) -> usize {
        self.header_size() + self.data.len()
    }
}

pub fn frag_udp_message(m: &UDPMessage, max_size: usize) -> Vec<UDPMessage> {
    if m.size() <= max_size {
        return vec![m.clone()];
    }

    let header_sz = m.header_size();
    if max_size <= header_sz {
        return vec![m.clone()];
    }

    let max_payload_size = max_size - header_sz;
    let full_payload = &m.data;
    let frag_count = full_payload.len().div_ceil(max_payload_size) as u8;

    let mut frags = Vec::with_capacity(frag_count as usize);
    let mut off = 0;
    let mut frag_id = 0u8;

    while off < full_payload.len() {
        let mut payload_size = full_payload.len() - off;
        if payload_size > max_payload_size {
            payload_size = max_payload_size;
        }

        let mut frag = m.clone();
        frag.frag_id = frag_id;
        frag.frag_count = frag_count;
        frag.data = full_payload[off..off + payload_size].to_vec();
        frags.push(frag);

        off += payload_size;
        frag_id += 1;
    }

    frags
}

pub struct Defragger {
    pkt_id: u16,
    frags: Vec<Option<UDPMessage>>,
    count: u8,
    size: usize,
}

impl Defragger {
    pub fn new() -> Self {
        Self {
            pkt_id: 0,
            frags: Vec::new(),
            count: 0,
            size: 0,
        }
    }

    pub fn feed(&mut self, m: UDPMessage) -> Option<UDPMessage> {
        if m.frag_count <= 1 {
            return Some(m);
        }
        if m.frag_id >= m.frag_count {
            return None;
        }

        if m.packet_id != self.pkt_id || m.frag_count as usize != self.frags.len() {
            self.pkt_id = m.packet_id;
            let mut frags = vec![None; m.frag_count as usize];
            self.size = m.data.len();
            self.count = 1;
            let fid = m.frag_id as usize;
            frags[fid] = Some(m);
            self.frags = frags;
        } else if self.frags[m.frag_id as usize].is_none() {
            self.count += 1;
            self.size += m.data.len();
            let frag_idx = m.frag_id as usize;
            self.frags[frag_idx] = Some(m);

            if self.count as usize == self.frags.len() {
                let mut data = Vec::with_capacity(self.size);
                let first = self.frags[0].as_ref().unwrap();
                let address = first.address.clone();
                let session_id = first.session_id;
                let packet_id = first.packet_id;

                for f in self.frags.iter().flatten() {
                    data.extend_from_slice(&f.data);
                }

                self.frags.clear();
                self.count = 0;
                self.size = 0;

                return Some(UDPMessage {
                    session_id,
                    packet_id,
                    frag_id: 0,
                    frag_count: 1,
                    address,
                    data,
                });
            }
        }
        None
    }
}

impl Default for Defragger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hysteria_frag_and_defrag() {
        let original_data = vec![0x42u8; 1000];
        let msg = UDPMessage::new(1, 100, "1.2.3.4:53".into(), original_data.clone());

        // Split into packets of max size 300
        let frags = frag_udp_message(&msg, 300);
        assert!(frags.len() > 1);

        let mut defragger = Defragger::new();
        let mut assembled = None;
        for frag in frags {
            if let Some(res) = defragger.feed(frag) {
                assembled = Some(res);
            }
        }

        assert!(assembled.is_some());
        let final_msg = assembled.unwrap();
        assert_eq!(final_msg.data, original_data);
        assert_eq!(final_msg.packet_id, 100);
    }
}
