pub mod byte;

#[cfg(test)]
pub mod byte_test;

pub use byte::Byte;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ByteMask(pub u8);

impl ByteMask {
    pub fn new(val: u8) -> Self {
        Self(val)
    }

    pub fn has(&self, other: ByteMask) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn set(&mut self, other: ByteMask) {
        self.0 |= other.0;
    }

    pub fn clear(&mut self, other: ByteMask) {
        self.0 &= !other.0;
    }

    pub fn toggle(&mut self, other: ByteMask) {
        self.0 ^= other.0;
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}
