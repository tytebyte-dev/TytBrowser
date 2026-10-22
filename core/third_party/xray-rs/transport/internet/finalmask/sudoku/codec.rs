// Module: transport\internet\finalmask\sudoku\codec.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\sudoku\codec.go

use super::table::{GoRand, SudokuTable, pack_key, sort4};
use crate::common::errors::{Error, Result};
use std::sync::Arc;

pub const PERM4: [[usize; 4]; 24] = [
    [0, 1, 2, 3],
    [0, 1, 3, 2],
    [0, 2, 1, 3],
    [0, 2, 3, 1],
    [0, 3, 1, 2],
    [0, 3, 2, 1],
    [1, 0, 2, 3],
    [1, 0, 3, 2],
    [1, 2, 0, 3],
    [1, 2, 3, 0],
    [1, 3, 0, 2],
    [1, 3, 2, 0],
    [2, 0, 1, 3],
    [2, 0, 3, 1],
    [2, 1, 0, 3],
    [2, 1, 3, 0],
    [2, 3, 0, 1],
    [2, 3, 1, 0],
    [3, 0, 1, 2],
    [3, 0, 2, 1],
    [3, 1, 0, 2],
    [3, 1, 2, 0],
    [3, 2, 0, 1],
    [3, 2, 1, 0],
];

pub struct SudokuCodec {
    pub tables: Vec<Arc<SudokuTable>>,
    pub rng: GoRand,
    pub padding_chance: usize,
    pub table_index: usize,
}

impl SudokuCodec {
    pub fn new(tables: Vec<Arc<SudokuTable>>, p_min: usize, p_max: usize) -> Self {
        let mut rng = GoRand::new_seeded();
        let padding_chance = pick_padding_chance(&mut rng, p_min, p_max);
        Self {
            tables,
            rng,
            padding_chance,
            table_index: 0,
        }
    }

    pub fn should_pad(&mut self) -> bool {
        if self.padding_chance == 0 {
            return false;
        }
        if self.padding_chance >= 100 {
            return true;
        }
        self.rng.intn(100) < self.padding_chance
    }

    pub fn current_table(&self) -> Option<Arc<SudokuTable>> {
        if self.tables.is_empty() {
            None
        } else {
            Some(self.tables[self.table_index % self.tables.len()].clone())
        }
    }

    pub fn random_padding(&mut self, t: &SudokuTable) -> u8 {
        let pool = &t.layout.padding_pool;
        pool[self.rng.intn(pool.len())]
    }

    pub fn encode(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let mut out = Vec::with_capacity(input.len() * 6 + 8);
        for &b in input {
            let t = self
                .current_table()
                .ok_or_else(|| Error::Config("sudoku table set missing".into()))?;
            if self.should_pad() {
                let pad = self.random_padding(&t);
                out.push(pad);
            }

            let enc = &t.encode[b as usize];
            if enc.is_empty() {
                return Err(Error::Config(format!(
                    "sudoku encode table missing for byte {}",
                    b
                )));
            }

            let hint_idx = self.rng.intn(enc.len());
            let hints = enc[hint_idx];
            let perm_idx = self.rng.intn(PERM4.len());
            let perm = PERM4[perm_idx];

            for idx in perm {
                if self.should_pad() {
                    let pad = self.random_padding(&t);
                    out.push(pad);
                }
                out.push(hints[idx]);
            }
            self.table_index += 1;
        }

        if self.should_pad()
            && let Some(t) = self.current_table()
        {
            let pad = self.random_padding(&t);
            out.push(pad);
        }

        Ok(out)
    }
}

pub fn pick_padding_chance(rng: &mut GoRand, mut p_min: usize, mut p_max: usize) -> usize {
    if p_max < p_min {
        p_max = p_min;
    }
    if p_min > 100 {
        p_min = 100;
    }
    if p_max > 100 {
        p_max = 100;
    }
    if p_max == p_min {
        return p_min;
    }
    p_min + rng.intn(p_max - p_min + 1)
}

pub fn decode_bytes(
    tables: &[Arc<SudokuTable>],
    table_index: &mut usize,
    input: &[u8],
    hint_buf: &mut Vec<u8>,
    out: &mut Vec<u8>,
) -> Result<()> {
    if tables.is_empty() {
        return Err(Error::Config("sudoku table set missing".into()));
    }

    for &b in input {
        let t = &tables[*table_index % tables.len()];
        if !t.layout.is_hint(b) {
            continue;
        }

        hint_buf.push(b);
        if hint_buf.len() < 4 {
            continue;
        }

        let key_bytes = sort4([hint_buf[0], hint_buf[1], hint_buf[2], hint_buf[3]]);
        let key = pack_key(key_bytes);
        let decoded = t
            .decode
            .get(&key)
            .copied()
            .ok_or_else(|| Error::Protocol("invalid sudoku hint tuple".into()))?;

        out.push(decoded);
        hint_buf.clear();
        *table_index += 1;
    }

    Ok(())
}
