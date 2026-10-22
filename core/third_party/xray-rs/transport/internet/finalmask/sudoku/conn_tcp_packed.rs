// Module: transport\internet\finalmask\sudoku\conn_tcp_packed.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\sudoku\conn_tcp_packed.go

use super::codec::SudokuCodec;
use super::table::{ByteLayout, SudokuTable, entropy_layout};
use crate::common::errors::{Error, Result};
use std::sync::Arc;

pub struct PackedEncoder {
    pub layouts: Vec<Arc<ByteLayout>>,
    pub codec: SudokuCodec,
    pub group_index: usize,
}

impl PackedEncoder {
    pub fn new(tables: &[Arc<SudokuTable>], p_min: usize, p_max: usize) -> Self {
        let mut layouts = Vec::with_capacity(tables.len());
        for t in tables {
            layouts.push(t.layout.clone());
        }
        if layouts.is_empty() {
            layouts.push(Arc::new(entropy_layout()));
        }

        Self {
            layouts,
            codec: SudokuCodec::new(Vec::new(), p_min, p_max),
            group_index: 0,
        }
    }

    pub fn encode(&mut self, p: &[u8]) -> Result<Vec<u8>> {
        let mut out = Vec::with_capacity(p.len() * 2 + 8);
        let mut bit_buf: u64 = 0;
        let mut bit_count: u8 = 0;

        for &b in p {
            bit_buf = (bit_buf << 8) | (b as u64);
            bit_count += 8;

            while bit_count >= 6 {
                bit_count -= 6;
                let layout = self.layouts[self.group_index % self.layouts.len()].clone();
                let group = (bit_buf >> bit_count) as u8;
                self.maybe_pad(&mut out, &layout);
                out.push((layout.encode_group)(group & 0x3f));
                self.group_index += 1;

                if bit_count > 0 {
                    bit_buf &= (1u64 << bit_count) - 1;
                } else {
                    bit_buf = 0;
                }
            }
        }

        if bit_count > 0 {
            let layout = self.layouts[self.group_index % self.layouts.len()].clone();
            let group = (bit_buf << (6 - bit_count)) as u8;
            self.maybe_pad(&mut out, &layout);
            out.push((layout.encode_group)(group & 0x3f));
            self.group_index += 1;

            let next_layout = self.layouts[self.group_index % self.layouts.len()].clone();
            out.push(next_layout.pad_marker);
        }

        let cur_layout = self.layouts[self.group_index % self.layouts.len()].clone();
        self.maybe_pad(&mut out, &cur_layout);
        Ok(out)
    }

    fn maybe_pad(&mut self, out: &mut Vec<u8>, layout: &ByteLayout) {
        if !self.codec.should_pad() {
            return;
        }
        if layout.padding_pool.len() == 1 {
            out.push(layout.padding_pool[0]);
            return;
        }
        loop {
            let b = layout.padding_pool[self.codec.rng.intn(layout.padding_pool.len())];
            if b != layout.pad_marker {
                out.push(b);
                break;
            }
        }
    }
}

pub struct PackedStreamDecoder {
    pub layouts: Vec<Arc<ByteLayout>>,
    pub group_index: usize,
    pub bit_buf: u64,
    pub bit_count: usize,
}

impl PackedStreamDecoder {
    pub fn new(tables: &[Arc<SudokuTable>]) -> Self {
        let mut layouts = Vec::with_capacity(tables.len());
        for t in tables {
            layouts.push(t.layout.clone());
        }
        if layouts.is_empty() {
            layouts.push(Arc::new(entropy_layout()));
        }

        Self {
            layouts,
            group_index: 0,
            bit_buf: 0,
            bit_count: 0,
        }
    }

    pub fn decode_chunk(&mut self, input: &[u8], pending: &mut Vec<u8>) -> Result<()> {
        let (buf, cnt, idx) = decode_packed_bytes(
            &self.layouts,
            input,
            self.bit_buf,
            self.bit_count,
            self.group_index,
            pending,
        )?;
        self.bit_buf = buf;
        self.bit_count = cnt;
        self.group_index = idx;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.bit_buf = 0;
        self.bit_count = 0;
    }
}

pub fn decode_packed_bytes(
    layouts: &[Arc<ByteLayout>],
    input: &[u8],
    mut bit_buf: u64,
    mut bit_count: usize,
    mut group_index: usize,
    out: &mut Vec<u8>,
) -> Result<(u64, usize, usize)> {
    if layouts.is_empty() {
        return Err(Error::Config("sudoku layout set missing".into()));
    }

    for &b in input {
        let layout = &layouts[group_index % layouts.len()];
        if !layout.is_hint(b) {
            if b == layout.pad_marker {
                bit_buf = 0;
                bit_count = 0;
            }
            continue;
        }

        let group = (layout.decode_group)(b)
            .ok_or_else(|| Error::Protocol(format!("invalid packed sudoku byte: {}", b)))?;
        group_index += 1;

        bit_buf = (bit_buf << 6) | (group as u64);
        bit_count += 6;

        while bit_count >= 8 {
            bit_count -= 8;
            out.push((bit_buf >> bit_count) as u8);
            if bit_count > 0 {
                bit_buf &= (1u64 << bit_count) - 1;
            } else {
                bit_buf = 0;
            }
        }
    }

    Ok((bit_buf, bit_count, group_index))
}
