//! Register file: 31 general-purpose 64-bit registers plus SP and PC.

use core::ops::{Index, IndexMut};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RegisterFile {
    x: [u64; 31],
    pub sp: u64,
    pub pc: u64,
}

impl RegisterFile {
    pub fn new() -> Self {
        Self::default()
    }

    /// Index must be 0-30. Panics otherwise.
    pub fn x(&self, index: u8) -> u64 {
        assert!(index < 31, "register index must be 0-30, got {}", index);
        self.x[index as usize]
    }

    /// Low 32 bits read. Zero-extends to u64 for return type, but caller casts down.
    pub fn w(&self, index: u8) -> u32 {
        self.x(index) as u32
    }

    pub fn set_x(&mut self, index: u8, value: u64) {
        assert!(index < 31, "register index must be 0-30, got {}", index);
        self.x[index as usize] = value;
    }

    /// Write low 32 bits, zero top 32 bits.
    pub fn set_w(&mut self, index: u8, value: u32) {
        self.set_x(index, value as u64);
    }
}

impl Index<u8> for RegisterFile {
    type Output = u64;
    fn index(&self, index: u8) -> &u64 {
        assert!(index < 31);
        &self.x[index as usize]
    }
}

impl IndexMut<u8> for RegisterFile {
    fn index_mut(&mut self, index: u8) -> &mut u64 {
        assert!(index < 31);
        &mut self.x[index as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn x_roundtrip() {
        let mut r = RegisterFile::new();
        r.set_x(5, 0xCAFE);
        assert_eq!(r.x(5), 0xCAFE);
    }

    #[test]
    fn w_zeros_top_half() {
        let mut r = RegisterFile::new();
        r.set_x(5, 0xDEADBEEF_CAFE0000);
        r.set_w(5, 0x12345678);
        assert_eq!(r.x(5), 0x12345678);
        assert_eq!(r.w(5), 0x12345678);
    }
}
