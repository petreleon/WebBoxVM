//! Flat physical memory: 1 GiB RAM at 0x4000_0000.

const RAM_BASE: u64 = 0x4000_0000;
const RAM_SIZE: usize = 1_073_741_824;
const RAM_TOP: u64 = RAM_BASE + RAM_SIZE as u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalMemory {
    ram: Vec<u8>,
}

impl PhysicalMemory {
    pub fn new() -> Self {
        Self {
            ram: vec![0u8; RAM_SIZE],
        }
    }

    pub fn read(&self, addr: u64, size: u8) -> Option<u64> {
        let offset = self.ram_offset(addr)?;
        Some(match size {
            1 => self.ram[offset] as u64,
            2 => u16::from_le_bytes([self.ram[offset], self.ram[offset + 1]]) as u64,
            4 => u32::from_le_bytes([
                self.ram[offset],
                self.ram[offset + 1],
                self.ram[offset + 2],
                self.ram[offset + 3],
            ]) as u64,
            8 => u64::from_le_bytes([
                self.ram[offset],
                self.ram[offset + 1],
                self.ram[offset + 2],
                self.ram[offset + 3],
                self.ram[offset + 4],
                self.ram[offset + 5],
                self.ram[offset + 6],
                self.ram[offset + 7],
            ]),
            _ => return None,
        })
    }

    pub fn write(&mut self, addr: u64, size: u8, value: u64) -> Option<()> {
        let offset = self.ram_offset(addr)?;
        match size {
            1 => self.ram[offset] = value as u8,
            2 => self.ram[offset..][..2].copy_from_slice(&(value as u16).to_le_bytes()),
            4 => self.ram[offset..][..4].copy_from_slice(&(value as u32).to_le_bytes()),
            8 => self.ram[offset..][..8].copy_from_slice(&value.to_le_bytes()),
            _ => return None,
        }
        Some(())
    }

    pub fn read_bytes(&self, addr: u64, dst: &mut [u8]) -> Option<()> {
        let offset = self.ram_offset(addr)?;
        dst.copy_from_slice(&self.ram[offset..offset + dst.len()]);
        Some(())
    }

    fn ram_offset(&self, addr: u64) -> Option<usize> {
        if addr >= RAM_BASE && addr < RAM_TOP {
            Some((addr - RAM_BASE) as usize)
        } else {
            None
        }
    }
}

impl Default for PhysicalMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u64_roundtrip() {
        let mut m = PhysicalMemory::new();
        assert!(m.write(0x4000_0000, 8, 0xCAFE0000_DEADBEEF).is_some());
        assert_eq!(m.read(0x4000_0000, 8), Some(0xCAFE0000_DEADBEEF));
    }

    #[test]
    fn u8_roundtrip() {
        let mut m = PhysicalMemory::new();
        assert!(m.write(0x4000_0100, 1, 0x42).is_some());
        assert_eq!(m.read(0x4000_0100, 1), Some(0x42));
    }

    #[test]
    fn unmapped_fails() {
        let m = PhysicalMemory::new();
        assert_eq!(m.read(0x0000_0000, 4), None);
    }
}
