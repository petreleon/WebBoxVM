use crate::constants::*;

mod sparse;

use sparse::SparseRegion;

/// Three disjoint sparse physical memory regions.
///
/// ```text
///   Low region  [0x0000_0000 .. 0x3FFF_FFFF)  1 GiB
///   RAM region  [0x4000_0000 .. 0x7FFF_FFFF)  1 GiB
///   EFI region  [0x8000_0000 .. 0x8FFF_FFFF)  256 MiB
/// ```
///
/// Writes that fall outside all three regions are silently discarded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalMemory {
    low: SparseRegion, // Low region: MMIO devices + reserved   (0x0         -> 0x3FFF_FFFF)
    ram: SparseRegion, // RAM region: kernel, stack, heap        (0x4000_0000 -> 0x7FFF_FFFF)
    efi: SparseRegion, // EFI region: firmware tables, services  (0x8000_0000 -> 0x8FFF_FFFF)
}

impl PhysicalMemory {
    pub fn new() -> Self {
        Self {
            low: SparseRegion::new(LOW_REGION_BASE, LOW_REGION_SIZE),
            ram: SparseRegion::new(RAM_BASE, RAM_SIZE),
            efi: SparseRegion::new(EFI_REGION_BASE, EFI_REGION_SIZE),
        }
    }

    pub fn read(&self, addr: u64, size: u8) -> Option<u64> {
        let mut bytes = [0u8; 8];
        let len = access_len(size)?;
        self.read_bytes(addr, &mut bytes[..len])?;
        Some(match size {
            1 => bytes[0] as u64,
            2 => u16::from_le_bytes([bytes[0], bytes[1]]) as u64,
            4 => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as u64,
            8 => u64::from_le_bytes(bytes),
            _ => return None,
        })
    }

    pub fn write(&mut self, addr: u64, size: u8, value: u64) -> Option<()> {
        let len = access_len(size)?;
        self.write_bytes(addr, &value.to_le_bytes()[..len])
    }

    /// Returns a pointer to guest RAM for JIT direct memory access.
    ///
    /// Sparse memory intentionally has no stable contiguous backing buffer. The
    /// current ARM64 JIT path is disabled before native blocks execute; keep a
    /// loud failure here so future JIT work does not accidentally use an invalid
    /// pointer.
    pub fn ram_data(&self) -> *const u8 {
        panic!("sparse guest memory has no contiguous RAM backing")
    }

    pub fn read_bytes(&self, addr: u64, dst: &mut [u8]) -> Option<()> {
        self.select_region(addr, dst.len())
            .and_then(|region| region.read_bytes(addr, dst))
    }

    pub fn write_bytes(&mut self, addr: u64, src: &[u8]) -> Option<()> {
        self.select_region_mut(addr, src.len())
            .and_then(|region| region.write_bytes(addr, src))
    }

    pub fn allocated_pages(&self) -> usize {
        self.low.allocated_pages() + self.ram.allocated_pages() + self.efi.allocated_pages()
    }

    fn select_region(&self, addr: u64, len: usize) -> Option<&SparseRegion> {
        if self.ram.contains_range(addr, len) {
            Some(&self.ram)
        } else if self.efi.contains_range(addr, len) {
            Some(&self.efi)
        } else if self.low.contains_range(addr, len) {
            Some(&self.low)
        } else {
            None
        }
    }

    fn select_region_mut(&mut self, addr: u64, len: usize) -> Option<&mut SparseRegion> {
        if self.ram.contains_range(addr, len) {
            Some(&mut self.ram)
        } else if self.efi.contains_range(addr, len) {
            Some(&mut self.efi)
        } else if self.low.contains_range(addr, len) {
            Some(&mut self.low)
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

fn access_len(size: u8) -> Option<usize> {
    match size {
        1 | 2 | 4 | 8 => Some(size as usize),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
