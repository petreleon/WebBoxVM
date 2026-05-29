//! Apply PE32+ base relocations (.reloc section).
//!
//! When a PE/COFF image is loaded at a different address than its
//! preferred base, every absolute pointer in the image must be adjusted
//! by the delta.  This module walks the .reloc directory and applies
//! these fixups.
//!
//! Supported relocation types:
//!   - IMAGE_REL_BASED_ABSOLUTE (0) — padding, no-op
//!   - IMAGE_REL_BASED_HIGHLOW  (3) — 32-bit fixup
//!   - IMAGE_REL_BASED_DIR64    (10) — 64-bit fixup

use crate::bus::SystemBus;
use crate::constants::*;

const IMAGE_REL_BASED_ABSOLUTE: u8 = 0;
const IMAGE_REL_BASED_HIGHLOW: u8 = 3;
const IMAGE_REL_BASED_DIR64: u8 = 10;

const RELOC_ENTRY_TY_MASK: u16 = 0xF000;
const RELOC_ENTRY_OFF_MASK: u16 = 0x0FFF;

#[derive(Debug)]
struct Pe32PlusInfo {
    preferred_base: u64,
    reloc_rva: usize,
    reloc_size: usize,
}

pub fn apply_pe_relocations(
    bus: &mut SystemBus,
    load_base: u64,
    data: &[u8],
) -> Result<(), &'static str> {
    let pe = parse_pe32_plus(data)?;
    if pe.reloc_rva == 0 || pe.reloc_size == 0 {
        return Ok(());
    }
    let delta = load_base.wrapping_sub(pe.preferred_base) as i64;
    if delta == 0 {
        return Ok(()); // loaded at preferred base — nothing to fix
    }
    apply_reloc_blocks(bus, load_base, data, pe.reloc_rva, pe.reloc_size, delta)
}

fn apply_reloc_blocks(
    bus: &mut SystemBus,
    load_base: u64,
    data: &[u8],
    start: usize,
    size: usize,
    delta: i64,
) -> Result<(), &'static str> {
    let end = start + size;
    if end > data.len() {
        return Err("reloc directory extends past file");
    }
    let mut off = start;
    while off + 8 <= end {
        let page_rva = read_u32(data, off)? as u64;
        let block_size = read_u32(data, off + 4)? as usize;
        if block_size == 0 { break; }
        if block_size < 8 || off + block_size > end {
            return Err("bad relocation block size");
        }
        let num_entries = (block_size - 8) / 2;
        for i in 0..num_entries {
            let entry = read_u16(data, off + 8 + i * 2)?;
            let ty = ((entry & RELOC_ENTRY_TY_MASK) >> 12) as u8;
            let offset = (entry & RELOC_ENTRY_OFF_MASK) as u64;
            let target = load_base + page_rva + offset;
            match ty {
                IMAGE_REL_BASED_ABSOLUTE => {} // padding
                IMAGE_REL_BASED_HIGHLOW  => apply_32bit_fixup(bus, target, delta)?,
                IMAGE_REL_BASED_DIR64    => apply_64bit_fixup(bus, target, delta)?,
                _ => return Err("unsupported PE relocation type"),
            }
        }
        off += block_size;
    }
    Ok(())
}

fn apply_32bit_fixup(bus: &mut SystemBus, target: u64, delta: i64) -> Result<(), &'static str> {
    let val = bus.read(target, 4).ok_or("reloc 32-bit read failed")? as u32;
    let fixed = (val as i64 + delta) as u32;
    bus.write(target, 4, fixed as u64);
    Ok(())
}

fn apply_64bit_fixup(bus: &mut SystemBus, target: u64, delta: i64) -> Result<(), &'static str> {
    let val = bus.read(target, 8).ok_or("reloc 64-bit read failed")?;
    let fixed = (val as i64).wrapping_add(delta) as u64;
    bus.write(target, 8, fixed);
    Ok(())
}

fn parse_pe32_plus(data: &[u8]) -> Result<Pe32PlusInfo, &'static str> {
    if data.len() < 0x44 {
        return Err("file too small for PE header");
    }
    // Read PE signature offset from the DOS header at offset 0x3C
    let pe_offset = read_u32(data, 0x3C)? as usize;
    if data.len() < pe_offset + 28 {
        return Err("PE header truncated");
    }
    if &data[pe_offset..pe_offset + 4] != PE_SIGNATURE.as_slice() {
        return Err("bad PE signature");
    }
    let coff_start = pe_offset + 4;
    let opt_start = coff_start + 20; // COFF header is 20 bytes
    if data.len() < opt_start + PE_OPT_HEADER_MIN_SIZE {
        return Err("optional header truncated");
    }
    let magic = read_u16(data, opt_start)?;
    if magic != PE32PLUS_MAGIC {
        return Err("not a PE32+ image");
    }
    let preferred_base = read_u64(data, opt_start + 24)?;
    let num_rva = read_u32(data, opt_start + 108)? as usize;
    if num_rva <= 5 {
        return Ok(Pe32PlusInfo { preferred_base, reloc_rva: 0, reloc_size: 0 });
    }
    // Data directories: 8 bytes each.  Entry 5 (0-indexed) is .reloc.
    let dd_start = opt_start + PE_OPT_HEADER_MIN_SIZE;
    let reloc_rva = read_u32(data, dd_start + 5 * 8)? as usize;
    let reloc_size = read_u32(data, dd_start + 5 * 8 + 4)? as usize;
    Ok(Pe32PlusInfo { preferred_base, reloc_rva, reloc_size })
}

// ── Little-endian read helpers ──

fn read_u16(data: &[u8], off: usize) -> Result<u16, &'static str> {
    data.get(off..off + 2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .ok_or("read_u16 oob")
}

fn read_u32(data: &[u8], off: usize) -> Result<u32, &'static str> {
    data.get(off..off + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or("read_u32 oob")
}

fn read_u64(data: &[u8], off: usize) -> Result<u64, &'static str> {
    data.get(off..off + 8)
        .map(|b| u64::from_le_bytes(b.try_into().unwrap()))
        .ok_or("read_u64 oob")
}

#[cfg(test)]
mod tests;
