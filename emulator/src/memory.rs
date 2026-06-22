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
    generation: u64,
}

impl PhysicalMemory {
    pub fn new() -> Self {
        Self {
            low: SparseRegion::new(LOW_REGION_BASE, LOW_REGION_SIZE),
            ram: SparseRegion::new(RAM_BASE, RAM_SIZE),
            efi: SparseRegion::new(EFI_REGION_BASE, EFI_REGION_SIZE),
            generation: 0,
        }
    }

    pub fn read(&self, addr: u64, size: u8) -> Option<u64> {
        Some(match size {
            1 => self.read_array::<1>(addr)?[0] as u64,
            2 => self.read_u16(addr)? as u64,
            4 => self.read_u32(addr)? as u64,
            8 => self.read_u64(addr)?,
            _ => return None,
        })
    }

    pub fn read_u16(&self, addr: u64) -> Option<u16> {
        Some(u16::from_le_bytes(self.read_array(addr)?))
    }

    pub fn read_u32(&self, addr: u64) -> Option<u32> {
        Some(u32::from_le_bytes(self.read_array(addr)?))
    }

    pub fn read_u64(&self, addr: u64) -> Option<u64> {
        Some(u64::from_le_bytes(self.read_array(addr)?))
    }

    pub fn write(&mut self, addr: u64, size: u8, value: u64) -> Option<()> {
        match size {
            1 => self.write_array(addr, [value as u8]),
            2 => self.write_array(addr, (value as u16).to_le_bytes()),
            4 => self.write_array(addr, (value as u32).to_le_bytes()),
            8 => self.write_array(addr, value.to_le_bytes()),
            _ => None,
        }
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
        let region = self.select_region(addr, dst.len())?;
        region.read_bytes_in_region(addr, dst);
        Some(())
    }

    pub fn write_bytes(&mut self, addr: u64, src: &[u8]) -> Option<()> {
        let region = self.select_region_mut(addr, src.len())?;
        region.write_bytes_in_region(addr, src);
        if !src.is_empty() {
            self.bump_generation();
        }
        Some(())
    }

    pub fn page_generation(&self, addr: u64) -> Option<u64> {
        self.select_region_for_addr(addr)
            .map(|region| region.page_generation(addr))
    }

    pub fn allocated_pages(&self) -> usize {
        self.low.allocated_pages() + self.ram.allocated_pages() + self.efi.allocated_pages()
    }

    pub fn generation(&self) -> u64 {
        self.generation
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

    fn select_region_for_addr(&self, addr: u64) -> Option<&SparseRegion> {
        if self.ram.contains_addr(addr) {
            Some(&self.ram)
        } else if self.efi.contains_addr(addr) {
            Some(&self.efi)
        } else if self.low.contains_addr(addr) {
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

    fn read_array<const N: usize>(&self, addr: u64) -> Option<[u8; N]> {
        self.select_region(addr, N)
            .map(|region| region.read_array(addr))
    }

    fn write_array<const N: usize>(&mut self, addr: u64, bytes: [u8; N]) -> Option<()> {
        let region = self.select_region_mut(addr, N)?;
        region.write_array(addr, bytes);
        self.bump_generation();
        Some(())
    }

    fn bump_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }
}

impl Default for PhysicalMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
