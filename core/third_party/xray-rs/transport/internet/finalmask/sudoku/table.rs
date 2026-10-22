// Module: transport\internet\finalmask\sudoku\table.rs
// 1:1 Rust implementation corresponding to Go transport\internet\finalmask\sudoku\table.go

use super::config::SudokuConfig;
use super::rng_cooked::RNG_COOKED;
use crate::common::errors::{Error, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

const RNG_LEN: usize = 607;
const RNG_TAP: usize = 273;
const RNG_MASK: u64 = (1 << 63) - 1;
const INT32_MAX: i32 = i32::MAX;

#[derive(Clone)]
pub struct GoRand {
    tap: usize,
    feed: usize,
    vec: [i64; RNG_LEN],
}

fn seedrand(mut x: i32) -> i32 {
    const A: i32 = 48271;
    const Q: i32 = 44488;
    const R: i32 = 3399;

    let hi = x / Q;
    let lo = x % Q;
    x = A * lo - R * hi;
    if x < 0 {
        x += INT32_MAX;
    }
    x
}

impl GoRand {
    pub fn new(mut seed: i64) -> Self {
        let tap = 0;
        let feed = RNG_LEN - RNG_TAP;

        seed %= INT32_MAX as i64;
        if seed < 0 {
            seed += INT32_MAX as i64;
        }
        if seed == 0 {
            seed = 89482311;
        }

        let mut x = seed as i32;
        let mut vec = [0i64; RNG_LEN];

        for i in -20..(RNG_LEN as isize) {
            x = seedrand(x);
            if i >= 0 {
                let mut u = (x as i64) << 40;
                x = seedrand(x);
                u ^= (x as i64) << 20;
                x = seedrand(x);
                u ^= x as i64;
                u ^= RNG_COOKED[i as usize];
                vec[i as usize] = u;
            }
        }

        Self { tap, feed, vec }
    }

    pub fn new_seeded() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;
        Self::new(nanos)
    }

    pub fn uint64(&mut self) -> u64 {
        if self.tap == 0 {
            self.tap = RNG_LEN - 1;
        } else {
            self.tap -= 1;
        }

        if self.feed == 0 {
            self.feed = RNG_LEN - 1;
        } else {
            self.feed -= 1;
        }

        let x = self.vec[self.feed].wrapping_add(self.vec[self.tap]);
        self.vec[self.feed] = x;
        x as u64
    }

    pub fn int63(&mut self) -> i64 {
        (self.uint64() & RNG_MASK) as i64
    }

    pub fn int31(&mut self) -> i32 {
        (self.int63() >> 32) as i32
    }

    pub fn int31n(&mut self, n: i32) -> i32 {
        if n <= 0 {
            return 0;
        }
        if (n & (n - 1)) == 0 {
            return self.int31() & (n - 1);
        }
        let max = ((1i64 << 31) - 1 - ((1i64 << 31) % (n as i64))) as i32;
        loop {
            let v = self.int31();
            if v <= max {
                return v % n;
            }
        }
    }

    pub fn intn(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        self.int31n(n as i32) as usize
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        let n = slice.len();
        for i in (1..n).rev() {
            let j = self.intn(i + 1);
            slice.swap(i, j);
        }
    }
}

pub type EncodeGroupFn = Arc<dyn Fn(u8) -> u8 + Send + Sync>;
pub type DecodeGroupFn = Arc<dyn Fn(u8) -> Option<u8> + Send + Sync>;

#[derive(Clone)]
pub struct ByteLayout {
    pub hint_mask: u8,
    pub hint_value: u8,
    pub pad_marker: u8,
    pub padding_pool: Vec<u8>,
    pub encode_hint: EncodeGroupFn,
    pub encode_group: EncodeGroupFn,
    pub decode_group: DecodeGroupFn,
}

impl ByteLayout {
    pub fn is_hint(&self, b: u8) -> bool {
        if (b & self.hint_mask) == self.hint_value {
            return true;
        }
        self.hint_mask == 0x40 && b == b'\n'
    }
}

pub fn ascii_layout() -> ByteLayout {
    let padding: Vec<u8> = (0..32).map(|i| 0x20 + i).collect();
    let encode_group: EncodeGroupFn = Arc::new(|group: u8| {
        let b = 0x40 | (group & 0x3f);
        if b == 0x7f { b'\n' } else { b }
    });

    let decode_group: DecodeGroupFn = Arc::new(|b: u8| {
        if b == b'\n' {
            return Some(0x3f);
        }
        if (b & 0x40) == 0 {
            return None;
        }
        Some(b & 0x3f)
    });

    ByteLayout {
        hint_mask: 0x40,
        hint_value: 0x40,
        pad_marker: 0x3f,
        padding_pool: padding,
        encode_hint: encode_group.clone(),
        encode_group,
        decode_group,
    }
}

pub fn entropy_layout() -> ByteLayout {
    let mut padding = Vec::with_capacity(16);
    for i in 0..8 {
        padding.push(0x80 + i);
        padding.push(0x10 + i);
    }

    let encode_group: EncodeGroupFn = Arc::new(|group: u8| {
        let v = group & 0x3f;
        ((v & 0x30) << 1) | (v & 0x0f)
    });

    let decode_group: DecodeGroupFn = Arc::new(|b: u8| {
        if (b & 0x90) != 0 {
            return None;
        }
        Some(((b >> 1) & 0x30) | (b & 0x0f))
    });

    ByteLayout {
        hint_mask: 0x90,
        hint_value: 0x00,
        pad_marker: 0x80,
        padding_pool: padding,
        encode_hint: encode_group.clone(),
        encode_group,
        decode_group,
    }
}

pub fn custom_layout(pattern: &str) -> Result<ByteLayout> {
    let pattern = SudokuConfig::normalize_custom_table(pattern)?;

    let mut x_bits = Vec::new();
    let mut p_bits = Vec::new();
    let mut v_bits = Vec::new();

    for (i, c) in pattern.chars().enumerate() {
        let bit = (7 - i) as u8;
        match c {
            'x' => x_bits.push(bit),
            'p' => p_bits.push(bit),
            'v' => v_bits.push(bit),
            _ => {}
        }
    }

    let mut x_mask: u8 = 0;
    for &bit in &x_bits {
        x_mask |= 1 << bit;
    }

    let x_bits_enc = x_bits.clone();
    let p_bits_enc = p_bits.clone();
    let v_bits_enc = v_bits.clone();
    let encode_with_drop = move |group: u8, drop_x: isize| -> u8 {
        let mut out = x_mask;
        if drop_x >= 0 && (drop_x as usize) < x_bits_enc.len() {
            out &= !(1 << x_bits_enc[drop_x as usize]);
        }

        let val = (group >> 4) & 0x03;
        let pos = group & 0x0f;

        if (val & 0x02) != 0 {
            out |= 1 << p_bits_enc[0];
        }
        if (val & 0x01) != 0 {
            out |= 1 << p_bits_enc[1];
        }
        for (i, &bit) in v_bits_enc.iter().enumerate() {
            if ((pos >> (3 - i)) & 0x01) == 1 {
                out |= 1 << bit;
            }
        }
        out
    };

    let mut padding = Vec::new();
    let mut padding_set = std::collections::HashSet::new();

    for drop in 0..x_bits.len() {
        for val in 0..4u8 {
            for pos in 0..16u8 {
                let group = (val << 4) | pos;
                let b = encode_with_drop(group, drop as isize);
                if b.count_ones() >= 5 && padding_set.insert(b) {
                    padding.push(b);
                }
            }
        }
    }
    padding.sort();
    if padding.is_empty() {
        return Err(Error::Config(
            "customTable produced empty padding pool".to_string(),
        ));
    }

    let p_bits_copy = p_bits.clone();
    let v_bits_copy = v_bits.clone();
    let decode_group: DecodeGroupFn = Arc::new(move |b: u8| {
        if (b & x_mask) != x_mask {
            return None;
        }

        let mut val = 0u8;
        let mut pos = 0u8;

        if (b & (1 << p_bits_copy[0])) != 0 {
            val |= 0x02;
        }
        if (b & (1 << p_bits_copy[1])) != 0 {
            val |= 0x01;
        }
        for (i, &bit) in v_bits_copy.iter().enumerate() {
            if (b & (1 << bit)) != 0 {
                pos |= 1 << (3 - i);
            }
        }

        Some(((val & 0x03) << 4) | (pos & 0x0f))
    });

    let p_bits_c = p_bits.clone();
    let v_bits_c = v_bits.clone();
    let encode_group: EncodeGroupFn = Arc::new(move |group: u8| {
        let mut out = x_mask;
        let val = (group >> 4) & 0x03;
        let pos = group & 0x0f;

        if (val & 0x02) != 0 {
            out |= 1 << p_bits_c[0];
        }
        if (val & 0x01) != 0 {
            out |= 1 << p_bits_c[1];
        }
        for (i, &bit) in v_bits_c.iter().enumerate() {
            if ((pos >> (3 - i)) & 0x01) == 1 {
                out |= 1 << bit;
            }
        }
        out
    });

    Ok(ByteLayout {
        hint_mask: x_mask,
        hint_value: x_mask,
        pad_marker: padding[0],
        padding_pool: padding,
        encode_hint: encode_group.clone(),
        encode_group,
        decode_group,
    })
}

pub fn sort4(mut in_b: [u8; 4]) -> [u8; 4] {
    if in_b[0] > in_b[1] {
        in_b.swap(0, 1);
    }
    if in_b[2] > in_b[3] {
        in_b.swap(2, 3);
    }
    if in_b[0] > in_b[2] {
        in_b.swap(0, 2);
    }
    if in_b[1] > in_b[3] {
        in_b.swap(1, 3);
    }
    if in_b[1] > in_b[2] {
        in_b.swap(1, 2);
    }
    in_b
}

pub fn pack_key(in_b: [u8; 4]) -> u32 {
    ((in_b[0] as u32) << 24) | ((in_b[1] as u32) << 16) | ((in_b[2] as u32) << 8) | (in_b[3] as u32)
}

fn generate_all_grids() -> Vec<[u8; 16]> {
    let mut grids = Vec::with_capacity(288);
    let mut g = [0u8; 16];

    fn dfs(idx: usize, g: &mut [u8; 16], grids: &mut Vec<[u8; 16]>) {
        if idx == 16 {
            grids.push(*g);
            return;
        }

        let row = idx / 4;
        let col = idx % 4;
        let box_row = (row / 2) * 2;
        let box_col = (col / 2) * 2;

        for num in 1..=4u8 {
            let mut valid = true;
            for i in 0..4 {
                if g[row * 4 + i] == num || g[i * 4 + col] == num {
                    valid = false;
                    break;
                }
            }
            if !valid {
                continue;
            }

            for r in 0..2 {
                for c in 0..2 {
                    if g[(box_row + r) * 4 + (box_col + c)] == num {
                        valid = false;
                        break;
                    }
                }
                if !valid {
                    break;
                }
            }
            if !valid {
                continue;
            }

            g[idx] = num;
            dfs(idx + 1, g, grids);
            g[idx] = 0;
        }
    }

    dfs(0, &mut g, &mut grids);
    grids
}

fn hint_positions() -> Vec<[u8; 4]> {
    let mut positions = Vec::with_capacity(1820);
    for a in 0..13 {
        for b in (a + 1)..14 {
            for c in (b + 1)..15 {
                for d in (c + 1)..16 {
                    positions.push([a as u8, b as u8, c as u8, d as u8]);
                }
            }
        }
    }
    positions
}

fn clue_group(g: &[u8; 16], pos: u8) -> u8 {
    ((g[pos as usize] - 1) << 4) | (pos & 0x0f)
}

fn build_base_patterns() -> Result<Vec<Vec<[u8; 4]>>> {
    let grids = generate_all_grids();
    let positions = hint_positions();

    let mut patterns = vec![Vec::new(); grids.len()];

    for ps in positions {
        let mut counts: HashMap<u32, u16> = HashMap::with_capacity(grids.len());
        let mut keys = Vec::with_capacity(grids.len());
        let mut groups_by_grid = Vec::with_capacity(grids.len());

        for g in &grids {
            let groups = sort4([
                clue_group(g, ps[0]),
                clue_group(g, ps[1]),
                clue_group(g, ps[2]),
                clue_group(g, ps[3]),
            ]);
            let key = pack_key(groups);
            keys.push(key);
            groups_by_grid.push(groups);
            *counts.entry(key).or_insert(0) += 1;
        }

        for (gi, &key) in keys.iter().enumerate() {
            if counts.get(&key) == Some(&1) {
                patterns[gi].push(groups_by_grid[gi]);
            }
        }
    }

    for (gi, list) in patterns.iter().enumerate() {
        if list.is_empty() {
            return Err(Error::Config(format!(
                "grid {} has no uniquely decodable clue set",
                gi
            )));
        }
    }

    Ok(patterns)
}

static BASE_PATTERNS: OnceLock<Vec<Vec<[u8; 4]>>> = OnceLock::new();

fn get_base_patterns() -> &'static [Vec<[u8; 4]>] {
    BASE_PATTERNS
        .get_or_init(|| build_base_patterns().expect("failed to build sudoku base patterns"))
}

#[derive(Clone)]
pub struct SudokuTable {
    pub encode: Vec<Vec<[u8; 4]>>,
    pub decode: HashMap<u32, u8>,
    pub layout: Arc<ByteLayout>,
}

impl SudokuTable {
    pub fn build(password: &str, layout: Arc<ByteLayout>) -> Result<Self> {
        let patterns = get_base_patterns();
        if patterns.len() < 256 {
            return Err(Error::Config(format!(
                "not enough sudoku grids: {}",
                patterns.len()
            )));
        }

        let mut order: Vec<usize> = (0..patterns.len()).collect();
        let hash = Sha256::digest(password.as_bytes());
        let seed = i64::from_be_bytes(hash[..8].try_into().unwrap());
        let mut rng = GoRand::new(seed);
        rng.shuffle(&mut order);

        let mut decode = HashMap::with_capacity(1 << 16);
        let mut encode = vec![Vec::new(); 256];

        for b in 0..256 {
            let pat_list = &patterns[order[b]];
            if pat_list.is_empty() {
                return Err(Error::Config(format!(
                    "grid {} has no valid clue set",
                    order[b]
                )));
            }

            let mut enc = Vec::with_capacity(pat_list.len());
            for groups in pat_list {
                let hints = [
                    (layout.encode_hint)(groups[0]),
                    (layout.encode_hint)(groups[1]),
                    (layout.encode_hint)(groups[2]),
                    (layout.encode_hint)(groups[3]),
                ];
                let sorted_hints = sort4(hints);
                let key = pack_key(sorted_hints);

                if let Some(&old) = decode.get(&key)
                    && old != b as u8
                {
                    return Err(Error::Config(format!(
                        "decode key collision for byte {} and {}",
                        old, b
                    )));
                }
                decode.insert(key, b as u8);
                enc.push(hints);
            }
            encode[b] = enc;
        }

        Ok(Self {
            encode,
            decode,
            layout,
        })
    }
}

static TABLE_SET_CACHE: OnceLock<Mutex<HashMap<String, Vec<Arc<SudokuTable>>>>> = OnceLock::new();

fn get_table_cache() -> &'static Mutex<HashMap<String, Vec<Arc<SudokuTable>>>> {
    TABLE_SET_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn resolve_layout(mode: &str, custom_table: &str) -> Result<ByteLayout> {
    if mode == "prefer_ascii" {
        return Ok(ascii_layout());
    }
    if !custom_table.is_empty() {
        return custom_layout(custom_table);
    }
    Ok(entropy_layout())
}

pub fn get_tables(config: &SudokuConfig) -> Result<Vec<Arc<SudokuTable>>> {
    let mode = config.normalized_ascii()?;
    let patterns = config.normalized_custom_patterns(&mode)?;

    let cache_key = format!("{}:{}:{}", config.password, mode, patterns.join("\0"));
    let cache = get_table_cache();
    {
        let guard = cache.lock().unwrap();
        if let Some(cached) = guard.get(&cache_key) {
            return Ok(cached.clone());
        }
    }

    let mut tables = Vec::with_capacity(patterns.len());
    for pattern in patterns {
        let layout = Arc::new(resolve_layout(&mode, &pattern)?);
        let table = Arc::new(SudokuTable::build(&config.password, layout)?);
        tables.push(table);
    }

    let mut guard = cache.lock().unwrap();
    guard.insert(cache_key, tables.clone());
    Ok(tables)
}
