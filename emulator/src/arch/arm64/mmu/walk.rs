use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WalkResult {
    pub pa: u64,
    pub desc: u64,
    pub desc_addr: u64,
}

impl WalkResult {
    fn new(pa: u64, desc: u64, desc_addr: u64) -> Self {
        Self {
            pa,
            desc,
            desc_addr,
        }
    }
}

pub(super) fn is_mmio_device_range(pa: u64) -> bool {
    (pa >= GICD_BASE && pa < GICD_BASE + GICD_SIZE) || (pa >= UART_BASE && pa < UART_END)
}

pub(super) fn page_table_walk(
    sys: &SystemRegisters,
    mem: &PhysicalMemory,
    va: u64,
) -> Result<u64, Fault> {
    page_table_walk_with_desc(sys, mem, va).map(|walk| walk.pa)
}

pub(super) fn page_table_walk_with_desc(
    sys: &SystemRegisters,
    mem: &PhysicalMemory,
    va: u64,
) -> Result<WalkResult, Fault> {
    let (mut table_base, start_level) = walk_root(sys, va);

    if start_level == 0 {
        table_base = descend_table(mem, table_base, va, PT_L0_SHIFT, 0)?;
    }

    if start_level <= 1 {
        let desc = read_level(mem, table_base, va, PT_L1_SHIFT, 1)?;
        if !desc.is_table {
            return Ok(WalkResult::new(
                (desc.raw & 0x0000_FFFF_C000_0000) | (va & (L1_BLOCK_SIZE - 1)),
                desc.raw,
                desc.addr,
            ));
        }
        table_base = desc.raw & DESC_ADDR_MASK;
    }

    if start_level <= 2 {
        let desc = read_level(mem, table_base, va, PT_L2_SHIFT, 2)?;
        if !desc.is_table {
            return Ok(WalkResult::new(
                (desc.raw & 0x0000_FFFF_FFE0_0000) | (va & (L2_BLOCK_SIZE - 1)),
                desc.raw,
                desc.addr,
            ));
        }
        table_base = desc.raw & DESC_ADDR_MASK;
    }

    let desc = read_level(mem, table_base, va, PT_L3_SHIFT, 3)?;
    if desc.is_table {
        return Err(Fault::TranslationFault);
    }
    Ok(WalkResult::new(
        (desc.raw & DESC_ADDR_MASK) | (va & PAGE_OFFSET_MASK),
        desc.raw,
        desc.addr,
    ))
}

fn walk_root(sys: &SystemRegisters, va: u64) -> (u64, u8) {
    let t1sz = ((sys.tcr_el1 >> TCR_T1SZ_SHIFT) & TCR_T1SZ_MASK) as u8;
    let va_bits = 64u8.saturating_sub(t1sz);
    let kernel_threshold = if va_bits >= 64 { 0 } else { (!0u64) << va_bits };
    let (ttbr, tnsz) = if va >= kernel_threshold {
        (sys.ttbr1_el1, t1sz)
    } else {
        (sys.ttbr0_el1, (sys.tcr_el1 & TCR_T0SZ_MASK) as u8)
    };
    (ttbr & DESC_ADDR_MASK, determine_start_level(tnsz))
}

fn descend_table(
    mem: &PhysicalMemory,
    table_base: u64,
    va: u64,
    shift: u64,
    level: u8,
) -> Result<u64, Fault> {
    let desc = read_level(mem, table_base, va, shift, level)?;
    if desc.is_table {
        Ok(desc.raw & DESC_ADDR_MASK)
    } else {
        Err(Fault::TranslationFault)
    }
}

#[derive(Debug, Clone, Copy)]
struct LevelDescriptor {
    raw: u64,
    addr: u64,
    is_table: bool,
}

fn read_level(
    mem: &PhysicalMemory,
    table_base: u64,
    va: u64,
    shift: u64,
    level: u8,
) -> Result<LevelDescriptor, Fault> {
    let idx = ((va >> shift) & 0x1FF) as u64;
    let addr = table_base + idx * 8;
    let raw = read_descriptor(mem, addr)?;
    Ok(LevelDescriptor {
        raw,
        addr,
        is_table: decode_descriptor_type(raw, level)?,
    })
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
    mem.read_u64(addr).ok_or(Fault::TranslationFault)
}

fn decode_descriptor_type(desc: u64, level: u8) -> Result<bool, Fault> {
    let low = desc & 3;
    if low == 0 {
        return Err(Fault::TranslationFault);
    }
    if level == 3 && low == 3 {
        return Ok(false);
    }
    Ok(low == 3)
}
