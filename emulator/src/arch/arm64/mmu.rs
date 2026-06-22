//! MMU: translates virtual addresses to physical addresses.
//!
//! The MMU performs a page-table walk for 4 KiB granules and maintains a small
//! direct-mapped software TLB to avoid repeated walks on hot pages.

use crate::arch::arm64::system_regs::SystemRegisters;
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
use tlb::{TlbContext, TlbInsert, descriptor_generation};
use walk::{is_mmio_device_range, page_table_walk_with_desc};

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
    translate_read(sys, Some(tlb), mem, va)
}

pub(crate) fn translate_read_only(
    sys: &SystemRegisters,
    mem: &PhysicalMemory,
    va: u64,
) -> Result<u64, Fault> {
    translate_read(sys, None, mem, va)
}

fn translate_read(
    sys: &SystemRegisters,
    mut tlb: Option<&mut Tlb>,
    mem: &PhysicalMemory,
    va: u64,
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

    let context = translation_context(sys, va);
    if let Some(tlb) = tlb.as_ref()
        && let Some(pa) = tlb.lookup(mem, va, context)
    {
        return Ok(pa);
    }

    let result = match page_table_walk_with_desc(sys, mem, va) {
        Ok(walk) => {
            if let Some(tlb) = tlb.as_mut()
                && let Some(meta) = tlb_insert(sys, mem, va, walk.desc_addr)
            {
                tlb.insert(
                    va,
                    walk.pa,
                    meta.context,
                    meta.desc_addr,
                    meta.desc_generation,
                );
            }
            Ok(walk.pa)
        }
        Err(Fault::TranslationFault) if va == 0 => Ok(0),
        Err(Fault::TranslationFault) if va >= KERNEL_VA_BASE => {
            let pa = va & VA_LOW32_MASK;
            if is_mmio_device_range(pa) {
                Ok(pa)
            } else {
                Err(Fault::TranslationFault)
            }
        }
        Err(e) => Err(e),
    };

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

    let context = translation_context(sys, va);
    if let Some(pa) = tlb.lookup_write(mem, va, current_el, context) {
        return Ok(pa);
    }

    let walk = page_table_walk_with_desc(sys, mem, va)?;
    trace_write_permission(sys, walk, va, current_el);
    check_write_permission(sys, mem, walk.desc_addr, walk.desc, current_el)?;
    if let Some(meta) = tlb_insert(sys, mem, va, walk.desc_addr) {
        tlb.insert_write(va, walk.pa, meta, (walk.desc & DESC_AP_EL0) != 0);
    }
    Ok(walk.pa)
}

fn tlb_insert(
    sys: &SystemRegisters,
    mem: &PhysicalMemory,
    va: u64,
    desc_addr: u64,
) -> Option<TlbInsert> {
    Some(TlbInsert {
        context: translation_context(sys, va),
        desc_addr,
        desc_generation: descriptor_generation(mem, desc_addr)?,
    })
}

fn translation_context(sys: &SystemRegisters, va: u64) -> TlbContext {
    let t1sz = ((sys.tcr_el1 >> TCR_T1SZ_SHIFT) & TCR_T1SZ_MASK) as u8;
    let va_bits = 64u8.saturating_sub(t1sz);
    let threshold = if va_bits >= 64 { 0 } else { (!0u64) << va_bits };
    let root = if va >= threshold {
        sys.ttbr1_el1
    } else {
        sys.ttbr0_el1
    };
    TlbContext {
        root,
        tcr: sys.tcr_el1,
    }
}

#[cfg(test)]
mod tests;
