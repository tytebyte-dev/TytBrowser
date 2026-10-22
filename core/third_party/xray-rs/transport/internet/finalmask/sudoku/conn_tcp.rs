// Module: transport\internet\finalmask\sudoku\conn_tcp.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\sudoku\conn_tcp.go

use super::codec::{SudokuCodec, decode_bytes};
use super::config::SudokuConfig;
use super::conn_tcp_packed::{PackedEncoder, PackedStreamDecoder};
use super::table::{SudokuTable, get_tables};
use crate::common::errors::Result;
use crate::transport::internet::finalmask::TcpMaskConn;
use std::sync::Arc;

pub struct HintStreamDecoder {
    pub tables: Vec<Arc<SudokuTable>>,
    pub table_index: usize,
    pub hint_buf: Vec<u8>,
}

impl HintStreamDecoder {
    pub fn new(tables: &[Arc<SudokuTable>]) -> Self {
        Self {
            tables: tables.to_vec(),
            table_index: 0,
            hint_buf: Vec::with_capacity(4),
        }
    }

    pub fn decode_chunk(&mut self, input: &[u8], pending: &mut Vec<u8>) -> Result<()> {
        decode_bytes(
            &self.tables,
            &mut self.table_index,
            input,
            &mut self.hint_buf,
            pending,
        )
    }

    pub fn reset(&mut self) {
        self.hint_buf.clear();
    }
}

pub struct SudokuTcpConn {
    pub tables: Vec<Arc<SudokuTable>>,
    pub is_client: bool,
    pub pure_encoder: SudokuCodec,
    pub pure_decoder: HintStreamDecoder,
    pub packed_encoder: PackedEncoder,
    pub packed_decoder: PackedStreamDecoder,
    pub pending_in: Vec<u8>,
}

impl SudokuTcpConn {
    pub fn new_client(config: &SudokuConfig) -> Result<Self> {
        Self::new(config, true)
    }

    pub fn new_server(config: &SudokuConfig) -> Result<Self> {
        Self::new(config, false)
    }

    pub fn new(config: &SudokuConfig, is_client: bool) -> Result<Self> {
        let tables = get_tables(config)?;
        let (p_min, p_max) = config.normalized_padding();

        let pure_encoder = SudokuCodec::new(tables.clone(), p_min, p_max);
        let pure_decoder = HintStreamDecoder::new(&tables);
        let packed_encoder = PackedEncoder::new(&tables, p_min, p_max);
        let packed_decoder = PackedStreamDecoder::new(&tables);

        Ok(Self {
            tables,
            is_client,
            pure_encoder,
            pure_decoder,
            packed_encoder,
            packed_decoder,
            pending_in: Vec::with_capacity(4096),
        })
    }

    /// Client writes pure sudoku; Server writes packed sudoku
    pub fn encode_stream(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        if self.is_client {
            self.pure_encoder.encode(data)
        } else {
            self.packed_encoder.encode(data)
        }
    }

    /// Client reads packed sudoku; Server reads pure sudoku
    pub fn decode_stream(&mut self, chunk: &[u8]) -> Result<Vec<u8>> {
        let mut decoded = Vec::new();
        if self.is_client {
            self.packed_decoder.decode_chunk(chunk, &mut decoded)?;
        } else {
            self.pure_decoder.decode_chunk(chunk, &mut decoded)?;
        }
        Ok(decoded)
    }

    /// Feeds raw incoming bytes and drains into user buffer
    pub fn feed_and_read(&mut self, raw: &[u8], user_buf: &mut [u8]) -> Result<usize> {
        if !raw.is_empty() {
            let decoded = self.decode_stream(raw)?;
            self.pending_in.extend_from_slice(&decoded);
        }

        let n = self.pending_in.len().min(user_buf.len());
        if n > 0 {
            user_buf[..n].copy_from_slice(&self.pending_in[..n]);
            self.pending_in.drain(..n);
        }
        Ok(n)
    }
}

impl TcpMaskConn for SudokuTcpConn {
    fn splice(&self) -> bool {
        // Sudoku transforms the entire stream; bypassing it would disable masking.
        false
    }
}
