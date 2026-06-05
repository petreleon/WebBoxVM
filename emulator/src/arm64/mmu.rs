//! MMU: translates virtual addresses to physical addresses.
//!
//! The MMU performs a page-table walk for 4 KiB granules and maintains a small
//! direct-mapped software TLB to avoid repeated walks on hot pages.

use crate::arm64::system_regs::SystemRegisters;
use crate::constants::*;
use crate::memory::PhysicalMemory;

mod debug;
mod permissions;
mod tlb;
mod walk;

#[allow(unused_imports)]
pub use debug::page_table_debug;
use permissions::{check_write_permission, trace_write_permission};
#[allow(unused_imports)]
pub use tlb::{Tlb, TlbEntry};
use walk::{is_mmio_device_range, page_table_walk, page_table_walk_with_desc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    TranslationFault,
    AccessFlagFault,
    PermissionFault,
}

/// Translate a virtual address to a physical address.
pub fn translate(
    sys: &SystemRegisters,
    tlb: &mut Tlb,
    mem: &PhysicalMemory,
    va: u64,
) -> Result<u64, Fault> {
    if (sys.sctlr_el1 & SCTLR_MMU_ENABLE) == 0 {
        return Ok(va);
    }

    if va >= KERNEL_VA_BASE {
        let low = va & VA_LOW32_MASK;
        if is_mmio_device_range(low) {
            tlb.insert(va, low);
            return Ok(low);
        }
    }

    if let Some(pa) = tlb.lookup(va) {
        return Ok(pa);
    }

    let result = match page_table_walk(sys, mem, va) {
        Ok(pa) => Ok(pa),
        Err(Fault::TranslationFault) if va == 0 => {
            tlb.insert(va, 0);
            Ok(0)
        }
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

pub fn translate_write(
    sys: &SystemRegisters,
    tlb: &mut Tlb,
    mem: &mut PhysicalMemory,
    va: u64,
    current_el: u8,
) -> Result<u64, Fault> {
    if (sys.sctlr_el1 & SCTLR_MMU_ENABLE) == 0 {
        return Ok(va);
    }

    if va >= KERNEL_VA_BASE {
        let low = va & VA_LOW32_MASK;
        if is_mmio_device_range(low) {
            return Ok(low);
        }
    }

    if let Some(pa) = tlb.lookup_write(va, current_el) {
        return Ok(pa);
    }

    let walk = page_table_walk_with_desc(sys, mem, va)?;
    trace_write_permission(sys, walk, va, current_el);
    check_write_permission(sys, mem, walk.desc_addr, walk.desc, current_el)?;
    tlb.insert_write(va, walk.pa, (walk.desc & DESC_AP_EL0) != 0);
    Ok(walk.pa)
}

#[cfg(test)]
mod tests;
