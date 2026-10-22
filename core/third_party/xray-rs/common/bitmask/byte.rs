// Module: common\bitmask\byte.rs
// 1:1 Rust implementation corresponding to Go common\bitmask\byte.go

/// Byte is a bitmask in byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Byte(pub u8);

impl Byte {
    pub fn new(val: u8) -> Self {
        Byte(val)
    }

    /// Has returns true if this bitmask contains another bitmask.
    pub fn has(&self, bb: impl Into<Byte>) -> bool {
        (self.0 & bb.into().0) != 0
    }

    pub fn set(&mut self, bb: impl Into<Byte>) {
        self.0 |= bb.into().0;
    }

    pub fn clear(&mut self, bb: impl Into<Byte>) {
        self.0 &= !bb.into().0;
    }

    pub fn toggle(&mut self, bb: impl Into<Byte>) {
        self.0 ^= bb.into().0;
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

impl From<u8> for Byte {
    fn from(val: u8) -> Self {
        Byte(val)
    }
}
