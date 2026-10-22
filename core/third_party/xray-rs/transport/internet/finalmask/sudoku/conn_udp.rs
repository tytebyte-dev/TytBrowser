// Module: transport\internet\finalmask\sudoku\conn_udp.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\sudoku\conn_udp.go

use super::codec::{SudokuCodec, decode_bytes};
use super::config::SudokuConfig;
use super::table::{SudokuTable, get_tables};
use crate::common::errors::{Error, Result};
use std::sync::Arc;

pub struct SudokuUdpConn {
    pub tables: Vec<Arc<SudokuTable>>,
    pub p_min: usize,
    pub p_max: usize,
}

impl SudokuUdpConn {
    pub fn new(config: &SudokuConfig) -> Result<Self> {
        let tables = get_tables(config)?;
        let (p_min, p_max) = config.normalized_padding();
        Ok(Self {
            tables,
            p_min,
            p_max,
        })
    }

    /// Obfuscates a single datagram.
    /// UDP decoding restarts at table 0 for every datagram, so encoding must do the same.
    pub fn mask(&self, payload: &[u8]) -> Result<Vec<u8>> {
        let mut codec = SudokuCodec::new(self.tables.clone(), self.p_min, self.p_max);
        codec.encode(payload)
    }

    /// Deobfuscates a single datagram.
    pub fn unmask(&self, packet: &[u8]) -> Result<Vec<u8>> {
        let mut decoded = Vec::with_capacity(packet.len() / 4 + 1);
        let mut hints = Vec::with_capacity(4);
        let mut table_index = 0;

        decode_bytes(
            &self.tables,
            &mut table_index,
            packet,
            &mut hints,
            &mut decoded,
        )?;

        if !hints.is_empty() {
            return Err(Error::Protocol("unexpected eof in sudoku datagram".into()));
        }

        Ok(decoded)
    }
}
