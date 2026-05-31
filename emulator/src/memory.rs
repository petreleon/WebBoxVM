use crate::constants::*;
use std::collections::HashMap;

const MEMORY_PAGE_SIZE: usize = PAGE_SIZE as usize;

type Page = Box<[u8; MEMORY_PAGE_SIZE]>;

/// Sparse physical memory region.
///
/// Guest address spaces are large, especially in browser builds.  Allocating the
/// full platform layout up front would reserve multiple GiB before Linux even
/// starts, so pages are materialized only after the guest or boot loader writes
/// to them.  Reads from untouched pages behave like zero-filled RAM.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SparseRegion {
    base: u64,
    size: u64,
    pages: HashMap<u64, Page>,
}

impl SparseRegion {
    fn new(base: u64, size: u64) -> Self {
        Self {
            base,
            size,
            pages: HashMap::new(),
        }
    }

    fn contains_range(&self, addr: u64, len: usize) -> bool {
        let Some(end) = addr.checked_add(len as u64) else {
            return false;
        };
        addr >= self.base && end <= self.base + self.size
    }

    fn read_bytes(&self, addr: u64, dst: &mut [u8]) -> Option<()> {
        if !self.contains_range(addr, dst.len()) {
            return None;
        }

        let mut done = 0usize;
        while done < dst.len() {
            let current = addr + done as u64;
            let offset = current - self.base;
            let page_index = offset / PAGE_SIZE;
            let page_offset = (offset % PAGE_SIZE) as usize;
            let chunk = (dst.len() - done).min(MEMORY_PAGE_SIZE - page_offset);

            if let Some(page) = self.pages.get(&page_index) {
                dst[done..done + chunk].copy_from_slice(&page[page_offset..page_offset + chunk]);
            } else {
                dst[done..done + chunk].fill(0);
            }
            done += chunk;
        }

        Some(())
    }

    fn write_bytes(&mut self, addr: u64, src: &[u8]) -> Option<()> {
        if !self.contains_range(addr, src.len()) {
            return None;
        }

        let mut done = 0usize;
        while done < src.len() {
            let current = addr + done as u64;
            let offset = current - self.base;
            let page_index = offset / PAGE_SIZE;
            let page_offset = (offset % PAGE_SIZE) as usize;
            let chunk = (src.len() - done).min(MEMORY_PAGE_SIZE - page_offset);
            let page = self
                .pages
                .entry(page_index)
                .or_insert_with(|| Box::new([0; MEMORY_PAGE_SIZE]));

            page[page_offset..page_offset + chunk].copy_from_slice(&src[done..done + chunk]);
            done += chunk;
        }

        Some(())
    }

    fn allocated_pages(&self) -> usize {
        self.pages.len()
    }
}

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
mod tests {
    use super::*;

    #[test]
    fn u64_roundtrip() {
        let mut m = PhysicalMemory::new();
        assert!(m.write(RAM_BASE, 8, 0xCAFE0000_DEADBEEF).is_some());
        assert_eq!(m.read(RAM_BASE, 8), Some(0xCAFE0000_DEADBEEF));
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

    #[test]
    fn new_memory_does_not_allocate_guest_pages() {
        let m = PhysicalMemory::new();

        assert_eq!(m.allocated_pages(), 0);
        assert_eq!(m.read(RAM_BASE + 0x1000, 8), Some(0));
        assert_eq!(m.allocated_pages(), 0);
    }

    #[test]
    fn bulk_access_crosses_sparse_pages() {
        let mut m = PhysicalMemory::new();
        let addr = RAM_BASE + PAGE_SIZE - 2;
        let bytes = [1, 2, 3, 4, 5];
        let mut out = [0u8; 5];

        m.write_bytes(addr, &bytes).unwrap();
        m.read_bytes(addr, &mut out).unwrap();

        assert_eq!(out, bytes);
        assert_eq!(m.read(addr, 4), Some(0x0403_0201));
    }

    #[test]
    fn range_must_stay_inside_one_region() {
        let mut m = PhysicalMemory::new();

        assert_eq!(m.write_bytes(LOW_REGION_END - 2, &[1, 2, 3]), None);
        assert_eq!(m.read(EFI_REGION_END, 1), None);
    }
}
