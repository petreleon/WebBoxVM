use crate::efi;

const RAM_BASE: u64 = 0x4000_0000;
const RAM_SIZE: usize = 1_073_741_824;
const RAM_TOP: u64 = RAM_BASE + RAM_SIZE as u64;

const EFI_SIZE: usize = efi::EFI_MEM_SIZE as usize;
const LOW_REGION_SIZE: usize = 0x4000_0000;

/// Three disjoint physical memory regions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalMemory {
    low: Vec<u8>, // 0x0         -> 0x3FFF_FFFF
    ram: Vec<u8>, // 0x4000_0000 -> 0x7FFF_FFFF
    efi: Vec<u8>, // 0x8000_0000 -> 0x8FFF_FFFF
}

impl PhysicalMemory {
    pub fn new() -> Self {
        Self {
            low: vec![0u8; LOW_REGION_SIZE],
            ram: vec![0u8; RAM_SIZE],
            efi: vec![0u8; EFI_SIZE],
        }
    }

    pub fn read(&self, addr: u64, size: u8) -> Option<u64> {
        self.select_region(addr)
            .and_then(|(bytes, offset)| read_bytes(bytes, offset, size))
    }

    pub fn write(&mut self, addr: u64, size: u8, value: u64) -> Option<()> {
        self.select_region_mut(addr)
            .and_then(|(bytes, offset)| write_bytes(bytes, offset, size, value))
    }

    /// Returns a pointer to guest RAM for JIT direct memory access.
    pub fn ram_data(&self) -> *const u8 {
        self.ram.as_ptr()
    }

    pub fn read_bytes(&self, addr: u64, dst: &mut [u8]) -> Option<()> {
        self.select_region(addr)
            .map(|(bytes, offset)| dst.copy_from_slice(&bytes[offset..offset + dst.len()]))
    }

    // --- region dispatch (RAM checked first to avoid low_region shadowing) ---

    fn select_region(&self, addr: u64) -> Option<(&[u8], usize)> {
        if let Some(o) = ram_offset(addr) { Some((&self.ram, o)) }
        else if let Some(o) = efi_offset(addr) { Some((&self.efi, o)) }
        else if let Some(o) = low_offset(addr) { Some((&self.low, o)) }
        else { None }
    }

    fn select_region_mut(&mut self, addr: u64) -> Option<(&mut [u8], usize)> {
        if let Some(o) = ram_offset(addr) { Some((&mut self.ram, o)) }
        else if let Some(o) = efi_offset(addr) { Some((&mut self.efi, o)) }
        else if let Some(o) = low_offset(addr) { Some((&mut self.low, o)) }
        else { None }
    }
}

impl Default for PhysicalMemory {
    fn default() -> Self { Self::new() }
}

// --- free functions (no duplication) ---

fn ram_offset(addr: u64) -> Option<usize> {
    if addr >= RAM_BASE && addr < RAM_TOP { Some((addr - RAM_BASE) as usize) } else { None }
}

fn efi_offset(addr: u64) -> Option<usize> {
    let base = efi::EFI_MEM_BASE;
    if addr >= base && addr < base + efi::EFI_MEM_SIZE { Some((addr - base) as usize) } else { None }
}

fn low_offset(addr: u64) -> Option<usize> {
    if addr < LOW_REGION_SIZE as u64 { Some(addr as usize) } else { None }
}

fn read_bytes(bytes: &[u8], offset: usize, size: u8) -> Option<u64> {
    match size {
        1 => Some(bytes[offset] as u64),
        2 => Some(u16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as u64),
        4 => Some(u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
        ]) as u64),
        8 => Some(u64::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ])),
        _ => None,
    }
}

fn write_bytes(bytes: &mut [u8], offset: usize, size: u8, value: u64) -> Option<()> {
    match size {
        1 => bytes[offset] = value as u8,
        2 => bytes[offset..][..2].copy_from_slice(&(value as u16).to_le_bytes()),
        4 => bytes[offset..][..4].copy_from_slice(&(value as u32).to_le_bytes()),
        8 => bytes[offset..][..8].copy_from_slice(&value.to_le_bytes()),
        _ => return None,
    }
    Some(())
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
    fn kernel_region_roundtrip() {
        let mut m = PhysicalMemory::new();
        assert!(m.write(0x1_0000, 8, 0x1234_5678_9ABC_DEFF).is_some());
        assert_eq!(m.read(0x1_0000, 8), Some(0x1234_5678_9ABC_DEFF));
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
        assert_eq!(m.read(0x0000_0000, 4), Some(0));
    }
}
