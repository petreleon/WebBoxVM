//! MMU: 3-level page table walk (39-bit VA) with software TLB.

use crate::arm64::system_regs::SystemRegisters;
use crate::memory::PhysicalMemory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    TranslationFault,
    AccessFlagFault,
    PermissionFault,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TlbEntry {
    pub valid: bool,
    pub va_page: u64,
    pub pa_page: u64,
}

impl Default for TlbEntry {
    fn default() -> Self {
        Self {
            valid: false,
            va_page: 0,
            pa_page: 0,
        }
    }
}

/// 2048-entry direct-mapped software TLB.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tlb {
    pub entries: Vec<TlbEntry>,
}

impl Tlb {
    pub fn new() -> Self {
        Self {
            entries: vec![TlbEntry::default(); 2048],
        }
    }

    pub fn lookup(&self, va: u64) -> Option<u64> {
        let page = va >> 12;
        let idx = (page & 0x7FF) as usize;
        let entry = &self.entries[idx];
        if entry.valid && entry.va_page == page {
            Some((entry.pa_page << 12) | (va & 0xFFF))
        } else {
            None
        }
    }

    pub fn insert(&mut self, va: u64, pa: u64) {
        let page = va >> 12;
        let idx = (page & 0x7FF) as usize;
        self.entries[idx] = TlbEntry {
            valid: true,
            va_page: page,
            pa_page: pa >> 12,
        };
    }

    pub fn invalidate_all(&mut self) {
        for entry in &mut self.entries {
            entry.valid = false;
        }
    }
}

impl Default for Tlb {
    fn default() -> Self {
        Self::new()
    }
}

/// Translate a virtual address to physical, using the TLB and page table walk.
pub fn translate(sys: &SystemRegisters, tlb: &mut Tlb, mem: &PhysicalMemory, va: u64) -> Result<u64, Fault> {
    // MMU off: pass through
    if (sys.sctlr_el1 & 1) == 0 {
        return Ok(va);
    }

    // TLB lookup
    if let Some(pa) = tlb.lookup(va) {
        return Ok(pa);
    }

    // Page table walk
    let pa = page_table_walk(sys, mem, va)?;
    tlb.insert(va, pa);
    Ok(pa)
}

fn page_table_walk(sys: &SystemRegisters, mem: &PhysicalMemory, va: u64) -> Result<u64, Fault> {
    let is_kernel = va >= 0xFFFF_FF80_0000_0000;
    let (ttbr, tnsz) = if is_kernel {
        (sys.ttbr1_el1, ((sys.tcr_el1 >> 16) & 0x3F) as u8)
    } else {
        (sys.ttbr0_el1, (sys.tcr_el1 & 0x3F) as u8)
    };

    let start_level = determine_start_level(tnsz);
    let mut table_base = ttbr & 0x0000_FFFF_FFFF_F000;

    // Level 0
    if start_level <= 0 {
        let idx = ((va >> 39) & 0x1FF) as u64;
        let desc = read_descriptor(mem, table_base + idx * 8)?;
        let (is_table, next_base) = decode_descriptor(desc, 0)?;
        if !is_table {
            return Err(Fault::TranslationFault);
        }
        table_base = next_base;
    }

    // Level 1
    if start_level <= 1 {
        let idx = ((va >> 30) & 0x1FF) as u64;
        let desc = read_descriptor(mem, table_base + idx * 8)?;
        let (is_table, next_base) = decode_descriptor(desc, 1)?;
        if !is_table {
            // L1 block: 1 GB
            return Ok((desc & 0x0000_FFFF_C000_0000) | (va & 0x3FFF_FFFF));
        }
        table_base = next_base;
    }

    // Level 2
    if start_level <= 2 {
        let idx = ((va >> 21) & 0x1FF) as u64;
        let desc = read_descriptor(mem, table_base + idx * 8)?;
        let (is_table, next_base) = decode_descriptor(desc, 2)?;
        if !is_table {
            // L2 block: 2 MB
            return Ok((desc & 0x0000_FFFF_FFE0_0000) | (va & 0x1F_FFFF));
        }
        table_base = next_base;
    }

    // Level 3: 4 KB page
    let idx = ((va >> 12) & 0x1FF) as u64;
    let desc = read_descriptor(mem, table_base + idx * 8)?;
    let (is_table, _) = decode_descriptor(desc, 3)?;
    if is_table {
        return Err(Fault::TranslationFault);
    }
    Ok((desc & 0x0000_FFFF_FFFF_F000) | (va & 0xFFF))
}

fn determine_start_level(tnsz: u8) -> u8 {
    match tnsz {
        34..=39 => 2,
        25..=33 => 1,
        16..=24 => 0,
        _ => 1,
    }
}

fn read_descriptor(mem: &PhysicalMemory, addr: u64) -> Result<u64, Fault> {
    mem.read(addr, 8).ok_or(Fault::TranslationFault)
}

fn decode_descriptor(desc: u64, _level: u8) -> Result<(bool, u64), Fault> {
    if (desc & 1) == 0 {
        return Err(Fault::TranslationFault);
    }
    let is_table = (desc & 2) != 0;
    let base = desc & 0x0000_FFFF_FFFF_F000;
    Ok((is_table, base))
}

#[cfg(test)]
mod tests;
