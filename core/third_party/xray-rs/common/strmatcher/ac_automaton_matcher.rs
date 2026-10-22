// Module: common\strmatcher\ac_automaton_matcher.rs
// 1:1 Rust implementation corresponding to Go common\strmatcher\ac_automaton_matcher.go

use super::strmatcher::MatcherType;
use std::collections::VecDeque;

const VALID_CHAR_COUNT: usize = 53;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchType {
    pub m_type: MatcherType,
    pub exist: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Edge {
    pub edge_type: bool, // true: TrieEdge, false: FailEdge
    pub next_node: usize,
}

pub struct AcAutomatonMatcher {
    trie: Vec<[Edge; VALID_CHAR_COUNT]>,
    fail: Vec<usize>,
    exists: Vec<MatchType>,
    count: usize,
}

fn char_to_index(c: u8) -> Option<usize> {
    match c {
        b'A'..=b'Z' => Some((c - b'A') as usize),
        b'a'..=b'z' => Some((c - b'a') as usize),
        b'!' => Some(26),
        b'$' => Some(27),
        b'&' => Some(28),
        b'\'' => Some(29),
        b'(' => Some(30),
        b')' => Some(31),
        b'*' => Some(32),
        b'+' => Some(33),
        b',' => Some(34),
        b';' => Some(35),
        b'=' => Some(36),
        b':' => Some(37),
        b'%' => Some(38),
        b'-' => Some(39),
        b'.' => Some(40),
        b'_' => Some(41),
        b'~' => Some(42),
        b'0'..=b'9' => Some(43 + (c - b'0') as usize),
        _ => None,
    }
}

impl AcAutomatonMatcher {
    pub fn new() -> Self {
        Self {
            trie: vec![[Edge::default(); VALID_CHAR_COUNT]],
            fail: vec![0],
            exists: vec![MatchType {
                m_type: MatcherType::Full,
                exist: false,
            }],
            count: 0,
        }
    }

    pub fn add(&mut self, domain: &str, t: MatcherType) {
        let bytes = domain.as_bytes();
        let mut node = 0;

        for &b in bytes.iter().rev() {
            let idx = match char_to_index(b) {
                Some(i) => i,
                None => return,
            };

            if self.trie[node][idx].next_node == 0 {
                self.count += 1;
                while self.trie.len() < self.count + 1 {
                    self.trie.push([Edge::default(); VALID_CHAR_COUNT]);
                    self.fail.push(0);
                    self.exists.push(MatchType {
                        m_type: MatcherType::Full,
                        exist: false,
                    });
                }
                self.trie[node][idx] = Edge {
                    edge_type: true,
                    next_node: self.count,
                };
            }
            node = self.trie[node][idx].next_node;
        }

        self.exists[node] = MatchType {
            m_type: t,
            exist: true,
        };

        if t == MatcherType::Domain {
            self.exists[node] = MatchType {
                m_type: MatcherType::Full,
                exist: true,
            };
            let dot_idx = char_to_index(b'.').unwrap();
            if self.trie[node][dot_idx].next_node == 0 {
                self.count += 1;
                while self.trie.len() < self.count + 1 {
                    self.trie.push([Edge::default(); VALID_CHAR_COUNT]);
                    self.fail.push(0);
                    self.exists.push(MatchType {
                        m_type: MatcherType::Full,
                        exist: false,
                    });
                }
                self.trie[node][dot_idx] = Edge {
                    edge_type: true,
                    next_node: self.count,
                };
            }
            node = self.trie[node][dot_idx].next_node;
            self.exists[node] = MatchType {
                m_type: t,
                exist: true,
            };
        }
    }

    pub fn build(&mut self) {
        let mut queue = VecDeque::new();
        for i in 0..VALID_CHAR_COUNT {
            if self.trie[0][i].next_node != 0 {
                queue.push_back(self.trie[0][i]);
            }
        }

        while let Some(front) = queue.pop_front() {
            let node = front.next_node;
            for i in 0..VALID_CHAR_COUNT {
                if self.trie[node][i].next_node != 0 {
                    let next = self.trie[node][i].next_node;
                    let fail_node = self.fail[node];
                    self.fail[next] = self.trie[fail_node][i].next_node;
                    queue.push_back(self.trie[node][i]);
                } else {
                    let fail_node = self.fail[node];
                    self.trie[node][i] = Edge {
                        edge_type: false,
                        next_node: self.trie[fail_node][i].next_node,
                    };
                }
            }
        }
    }

    pub fn match_str(&self, s: &str) -> bool {
        let bytes = s.as_bytes();
        let mut node = 0;
        let mut full_match = true;

        for &b in bytes.iter().rev() {
            let idx = match char_to_index(b) {
                Some(i) => i,
                None => return false,
            };

            full_match = full_match && self.trie[node][idx].edge_type;
            node = self.trie[node][idx].next_node;

            match self.exists[node].m_type {
                MatcherType::Substr => return true,
                MatcherType::Domain if full_match => {
                    return true;
                }
                _ => {}
            }
        }

        full_match && self.exists[node].exist
    }
}

impl Default for AcAutomatonMatcher {
    fn default() -> Self {
        Self::new()
    }
}
