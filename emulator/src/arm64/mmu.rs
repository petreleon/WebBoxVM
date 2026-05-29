//! MMU — Memory Management Unit: translates virtual addresses to physical.
//!
//! The MMU performs a 3-level page-table walk (39-bit VA, 4 KiB granule).
//! It also includes a software TLB (2048 entries, direct-mapped) to cache
//! recent translations and avoid walking the page tables every instruction.
//!
//! ## How the page table walk works
//!
//! A 48-bit virtual address is split into:
//!   bits [47:39] → Level 0 index (512 entries)
//!   bits [38:30] → Level 1 index (512 entries)
//!   bits [29:21] → Level 2 index (512 entries)
//!   bits [20:12] → Level 3 index (512 entries)
//!   bits [11:0]  → in-page offset (4 KiB)
//!
//! At each level we read an 8-byte descriptor.  If it points to another table
//! we continue; if it's a block/page we extract the physical address; if it's
//! invalid we raise a Translation Fault.

use crate::constants::*;
use crate::arm64::system_regs::SystemRegisters;
use crate::memory::PhysicalMemory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    TranslationFault,
    AccessFlagFault,
    PermissionFault,
}

/// A single TLB (Translation Lookaside Buffer) entry.
/// Caches one VA→PA mapping at page granularity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TlbEntry {
    pub valid: bool,
    pub va_page: u64,   // virtual page number (VA >> 12)
    pub pa_page: u64,   // physical page number (PA >> 12)
}

impl Default for TlbEntry {
    fn default() -> Self {
        Self { valid: false, va_page: 0, pa_page: 0 }
    }
}

/// Direct-mapped software TLB with TLB_ENTRIES slots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tlb {
    pub entries: Vec<TlbEntry>,
}

impl Tlb {
    pub fn new() -> Self {
        Self { entries: vec![TlbEntry::default(); TLB_ENTRIES] }
    }

    /// Look up a virtual address in the TLB.
    /// Returns the corresponding physical address if found and valid.
    pub fn lookup(&self, va: u64) -> Option<u64> {
        let page = va >> PAGE_SHIFT;
        let idx = (page & TLB_INDEX_MASK) as usize;
        let entry = &self.entries[idx];
        if entry.valid && entry.va_page == page {
            Some((entry.pa_page << PAGE_SHIFT) | (va & PAGE_OFFSET_MASK))
        } else {
            None
        }
    }

    /// Insert a VA→PA mapping into the TLB.
    pub fn insert(&mut self, va: u64, pa: u64) {
        let page = va >> PAGE_SHIFT;
        let idx = (page & TLB_INDEX_MASK) as usize;
        self.entries[idx] = TlbEntry {
            valid: true,
            va_page: page,
            pa_page: pa >> PAGE_SHIFT,
        };
    }

    /// Invalidate (flush) the entire TLB — called on TLBI instructions.
    pub fn invalidate_all(&mut self) {
        for entry in &mut self.entries {
            entry.valid = false;
        }
    }
}

impl Default for Tlb {
    fn default() -> Self { Self::new() }
}

/// Translate a virtual address to a physical address.
///
/// Steps:
///   1. If the MMU is off (SCTLR_EL1.M == 0), pass through VA as PA.
///   2. Check the TLB for a cached translation.
///   3. If not cached, walk the page tables.
///   4. Cache the result in the TLB.
pub fn translate(
    sys: &SystemRegisters,
    tlb: &mut Tlb,
    mem: &PhysicalMemory,
    va: u64,
) -> Result<u64, Fault> {
    // MMU disabled → identity map (VA = PA)
    if (sys.sctlr_el1 & SCTLR_MMU_ENABLE) == 0 {
        return Ok(va);
    }

    // Force identity-map for kernel VAs targeting known MMIO devices.
    // This fixes early_ioremap fixmap entries that map to wrong PAs
    // due to page-table entry OA computation differences.
    if va >= KERNEL_VA_BASE {
        let low = va & VA_LOW32_MASK;
        if is_mmio_device_range(low) {
            tlb.insert(va, low);
            return Ok(low);
        }
    }

    // TLB lookup
    if let Some(pa) = tlb.lookup(va) {
        return Ok(pa);
    }

    // Page table walk
    let result = match page_table_walk(sys, mem, va) {
        Ok(pa) => Ok(pa),
        // Gracefully handle translation faults on null pointer (0x0)
        Err(Fault::TranslationFault) if va == 0 => {
            tlb.insert(va, 0);
            Ok(0)
        }
        // Fallback for kernel VAs: try identity map on MMIO devices
        Err(Fault::TranslationFault) if va >= KERNEL_VA_BASE => {
            let pa = va & VA_LOW32_MASK;
            if is_mmio_device_range(pa) {
                tlb.insert(va, pa);
                Ok(pa)
            } else {
                Err(Fault::TranslationFault)
            }
        }
        Err(e) => Err(e),
    };

    if let Ok(pa) = result {
        tlb.insert(va, pa);
    }
    result
}

/// Returns true if the 32-bit physical address is a known MMIO device.
fn is_mmio_device_range(pa: u64) -> bool {
    (pa >= GICD_BASE && pa < GICD_BASE + GICD_SIZE)
        || (pa >= UART_BASE && pa < UART_END)
}

/// Walk the 3-level page table structure to translate a VA.
fn page_table_walk(
    sys: &SystemRegisters,
    mem: &PhysicalMemory,
    va: u64,
) -> Result<u64, Fault> {
    // Determine which TTBR to use and the VA size
    let t1sz = ((sys.tcr_el1 >> TCR_T1SZ_SHIFT) & TCR_T1SZ_MASK) as u8;
    let va_bits = 64u8.saturating_sub(t1sz);
    let kernel_threshold = if va_bits >= 64 { 0 } else { (!0u64) << va_bits };

    let is_kernel = va >= kernel_threshold;
    let (ttbr, tnsz) = if is_kernel {
        (sys.ttbr1_el1, t1sz)
    } else {
        (sys.ttbr0_el1, (sys.tcr_el1 & TCR_T0SZ_MASK) as u8)
    };

    let start_level = determine_start_level(tnsz);
    let mut table_base = ttbr & DESC_ADDR_MASK;

    // Level 0 (bits 47:39)
    if start_level <= 0 {
        let idx = ((va >> PT_L0_SHIFT) & 0x1FF) as u64;
        let desc = read_descriptor(mem, table_base + idx * 8)?;
        let is_table = decode_descriptor_type(desc, 0)?;
        if !is_table {
            return Err(Fault::TranslationFault);
        }
        table_base = desc & DESC_ADDR_MASK;
    }

    // Level 1 (bits 38:30) — can be a 1 GiB block
    if start_level <= 1 {
        let idx = ((va >> PT_L1_SHIFT) & 0x1FF) as u64;
        let desc = read_descriptor(mem, table_base + idx * 8)?;
        let is_table = decode_descriptor_type(desc, 1)?;
        if !is_table {
            return Ok((desc & 0x0000_FFFF_C000_0000) | (va & (L1_BLOCK_SIZE - 1)));
        }
        table_base = desc & DESC_ADDR_MASK;
    }

    // Level 2 (bits 29:21) — can be a 2 MiB block
    if start_level <= 2 {
        let idx = ((va >> PT_L2_SHIFT) & 0x1FF) as u64;
        let desc = read_descriptor(mem, table_base + idx * 8)?;
        let is_table = decode_descriptor_type(desc, 2)?;
        if !is_table {
            return Ok((desc & 0x0000_FFFF_FFE0_0000) | (va & (L2_BLOCK_SIZE - 1)));
        }
        table_base = desc & DESC_ADDR_MASK;
    }

    // Level 3 (bits 20:12) — 4 KiB page
    let idx = ((va >> PT_L3_SHIFT) & 0x1FF) as u64;
    let desc = read_descriptor(mem, table_base + idx * 8)?;
    let is_table = decode_descriptor_type(desc, 3)?;
    if is_table {
        return Err(Fault::TranslationFault); // L3 can't point to a sub-table
    }
    Ok((desc & DESC_ADDR_MASK) | (va & PAGE_OFFSET_MASK))
}

/// Determine which page table level to start at based on the VA size.
///
/// | TCR.TxSZ    | VA size  | Start level |
/// |-------------|----------|-------------|
/// | 34..39      | 25..30   | 2           |
/// | 25..33      | 31..39   | 1           |
/// | 16..24      | 40..48   | 0           |
fn determine_start_level(tnsz: u8) -> u8 {
    match tnsz {
        34..=39 => 2,  // short VA → skip L0 and L1
        25..=33 => 1,  // medium VA → skip L0
        16..=24 => 0,  // full 48-bit VA → start at L0
        _ => 1,        // default
    }
}

/// Read an 8-byte page table descriptor from physical memory.
fn read_descriptor(mem: &PhysicalMemory, addr: u64) -> Result<u64, Fault> {
    mem.read(addr, 8).ok_or(Fault::TranslationFault)
}

/// Debug: walk page table from TTBR1 and print each step.
pub fn page_table_debug(sys: &SystemRegisters, mem: &PhysicalMemory, va: u64) {
    let ttbr = sys.ttbr1_el1 & DESC_ADDR_MASK;
    let tcr = sys.tcr_el1;
    let t1sz = ((tcr >> TCR_T1SZ_SHIFT) & TCR_T1SZ_MASK) as u8;
    let va_bits = 64u8.saturating_sub(t1sz);

    eprintln!("    TTBR1=0x{:016x}  TCR=0x{:016x}  T1SZ={}  VA_BITS={}", ttbr, tcr, t1sz, va_bits);
    eprintln!("    VA =0x{:016x}", va);

    // L0
    let l0_idx = ((va >> PT_L0_SHIFT) & 0x1FF) as u64;
    let l0_addr = ttbr + l0_idx * 8;
    if let Some(desc) = mem.read(l0_addr, 8) {
        let valid = desc & 3;
        eprintln!("    L0[{}] at PA=0x{:x} desc=0x{:016x} valid={}", l0_idx, l0_addr, desc, valid);
        if valid == DESC_TABLE {
            let l1_base = desc & DESC_ADDR_MASK;
            // L1
            let l1_idx = ((va >> PT_L1_SHIFT) & 0x1FF) as u64;
            let l1_addr = l1_base + l1_idx * 8;
            if let Some(desc) = mem.read(l1_addr, 8) {
                let valid = desc & 3;
                eprintln!("    L1[{}] at PA=0x{:x} desc=0x{:016x} valid={}", l1_idx, l1_addr, desc, valid);
                if valid == DESC_TABLE {
                    let l2_base = desc & DESC_ADDR_MASK;
                    // L2
                    let l2_idx = ((va >> PT_L2_SHIFT) & 0x1FF) as u64;
                    let l2_addr = l2_base + l2_idx * 8;
                    if let Some(desc) = mem.read(l2_addr, 8) {
                        let valid = desc & 3;
                        eprintln!("    L2[{}] at PA=0x{:x} desc=0x{:016x} valid={}", l2_idx, l2_addr, desc, valid);
                        if valid == DESC_TABLE {
                            let l3_base = desc & DESC_ADDR_MASK;
                            // L3
                            let l3_idx = ((va >> PT_L3_SHIFT) & 0x1FF) as u64;
                            let l3_addr = l3_base + l3_idx * 8;
                            if let Some(desc) = mem.read(l3_addr, 8) {
                                let pa = (desc & DESC_ADDR_MASK) | (va & PAGE_OFFSET_MASK);
                                eprintln!("    L3[{}] at PA=0x{:x} desc=0x{:016x} -> PA=0x{:016x}", l3_idx, l3_addr, desc, pa);
                            } else { eprintln!("    L3[{}] at PA=0x{:x} UNREADABLE", l3_idx, l3_addr); }
                        } else if valid == DESC_BLOCK {
                            let pa = (desc & 0x0000_FFFF_FFE0_0000) | (va & (L2_BLOCK_SIZE - 1));
                            eprintln!("    L2 block -> PA=0x{:016x}", pa);
                        } else { eprintln!("    L2 INVALID (valid={})", valid); }
                    } else { eprintln!("    L2[{}] at PA=0x{:x} UNREADABLE", l2_idx, l2_addr); }
                } else if valid == DESC_BLOCK {
                    let pa = (desc & 0x0000_FFFF_C000_0000) | (va & (L1_BLOCK_SIZE - 1));
                    eprintln!("    L1 block -> PA=0x{:016x}", pa);
                } else { eprintln!("    L1 INVALID (valid={})", valid); }
            } else { eprintln!("    L1[{}] at PA=0x{:x} UNREADABLE", l1_idx, l1_addr); }
        } else if valid == DESC_BLOCK {
            let pa = (desc & 0x0000_FFFF_FFFF_F000) | (va & (L0_BLOCK_SIZE - 1));
            eprintln!("    L0 block -> PA=0x{:016x}", pa);
        } else { eprintln!("    L0 INVALID (valid={})", valid); }
    } else { eprintln!("    L0[{}] at PA=0x{:x} UNREADABLE", l0_idx, l0_addr); }
}

/// Decode a page table descriptor at `level`.
/// Returns `(is_table_pointer, _)` — true if descriptor points to another table.
///
/// Descriptor bits:
///   [1:0] = 0b00 → invalid (translation fault)
///   [1:0] = 0b01 → block/page descriptor (at L1/L2) or page (at L3)
///   [1:0] = 0b11 → table descriptor (points to next level; at L3 means page)
fn decode_descriptor_type(desc: u64, level: u8) -> Result<bool, Fault> {
    let low = desc & 3;
    if low == 0 { return Err(Fault::TranslationFault); }
    let is_table = low == 3; // 0b11 = table pointer
    // At L3, 0b11 means a 4 KiB page, not a table
    if level == 3 && is_table {
        return Ok(false);
    }
    Ok(is_table)
}

#[cfg(test)]
mod tests;
