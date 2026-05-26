//! Apply PE32+ base relocations (.reloc section).

use crate::bus::SystemBus;

const MAGIC_PE32PLUS: u16 = 0x20B;

#[derive(Debug)]
struct Pe32Plus {
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
        return Ok(());
    }
    apply_blocks(bus, load_base, data, pe.reloc_rva, pe.reloc_size, delta)
}

fn apply_blocks(
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
        let block_sz = read_u32(data, off + 4)? as usize;
        if block_sz == 0 {
            break;
        }
        if block_sz < 8 || off + block_sz > end {
            return Err("bad relocation block size");
        }
        let num = (block_sz - 8) / 2;
        for i in 0..num {
            let entry = read_u16(data, off + 8 + i * 2)?;
            let ty = (entry >> 12) as u8;
            let offset = (entry & 0x0FFF) as u64;
            let target = load_base + page_rva + offset;
            match ty {
                0 => {} // IMAGE_REL_BASED_ABSOLUTE – padding
                3 => apply_32(bus, target, delta)?,
                10 => apply_64(bus, target, delta)?,
                _ => return Err("unsupported PE relocation type"),
            }
        }
        off += block_sz;
    }
    Ok(())
}

fn apply_32(bus: &mut SystemBus, target: u64, delta: i64) -> Result<(), &'static str> {
    let val = bus.read(target, 4).ok_or("reloc 32-bit read failed")? as u32;
    let fixed = (val as i64 + delta) as u32;
    bus.write(target, 4, fixed as u64);
    Ok(())
}

fn apply_64(bus: &mut SystemBus, target: u64, delta: i64) -> Result<(), &'static str> {
    let val = bus.read(target, 8).ok_or("reloc 64-bit read failed")?;
    let fixed = (val as i64).wrapping_add(delta) as u64;
    bus.write(target, 8, fixed);
    Ok(())
}

fn parse_pe32_plus(data: &[u8]) -> Result<Pe32Plus, &'static str> {
    if data.len() < 0x44 {
        return Err("file too small for PE header");
    }
    let pe_offset = read_u32(data, 0x3C)? as usize;
    if data.len() < pe_offset + 28 {
        return Err("PE header truncated");
    }
    if &data[pe_offset..pe_offset + 4] != b"PE\0\0" {
        return Err("bad PE signature");
    }
    let coff_start = pe_offset + 4;
    let opt_start = coff_start + 20;
    if data.len() < opt_start + 112 {
        return Err("optional header truncated");
    }
    let magic = read_u16(data, opt_start)?;
    if magic != MAGIC_PE32PLUS {
        return Err("not a PE32+ image");
    }
    let preferred_base = read_u64(data, opt_start + 24)?;
    let num_rva = read_u32(data, opt_start + 108)? as usize;
    if num_rva <= 5 {
        return Ok(Pe32Plus {
            preferred_base,
            reloc_rva: 0,
            reloc_size: 0,
        });
    }
    let dd_start = opt_start + 112;
    let reloc_rva = read_u32(data, dd_start + 5 * 8)? as usize;
    let reloc_size = read_u32(data, dd_start + 5 * 8 + 4)? as usize;
    Ok(Pe32Plus {
        preferred_base,
        reloc_rva,
        reloc_size,
    })
}

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
mod tests {
    use super::*;
    use crate::bus::SystemBus;

    fn fake_pe32_plus(reloc_rva: u32, reloc_size: u32, preferred_base: u64) -> Vec<u8> {
        let mut data = vec![0u8; 0x200];
        // ARM64 kernel magic
        data[56..60].copy_from_slice(&0x644d5241u32.to_le_bytes());
        // e_lfanew at 0x3C
        data[0x3C..0x40].copy_from_slice(&0x40u32.to_le_bytes());
        // PE signature
        data[0x40..0x44].copy_from_slice(b"PE\0\0");
        // COFF header: 20 bytes
        let opt_start = 0x40 + 4 + 20;

        // Optional header magic
        data[opt_start..opt_start + 2].copy_from_slice(&MAGIC_PE32PLUS.to_le_bytes());
        // AddressOfEntryPoint skip
        // ImageBase at opt_start+24
        data[opt_start + 24..opt_start + 32].copy_from_slice(&preferred_base.to_le_bytes());
        // NumberOfRvaAndSizes at opt_start+108
        data[opt_start + 108..opt_start + 112].copy_from_slice(&6u32.to_le_bytes());
        // DataDirectory[5] at opt_start+112+5*8
        let dd5 = opt_start + 112 + 5 * 8;
        data[dd5..dd5 + 4].copy_from_slice(&reloc_rva.to_le_bytes());
        data[dd5 + 4..dd5 + 8].copy_from_slice(&reloc_size.to_le_bytes());

        // Place relocation block at reloc_rva
        let block_rva = reloc_rva as usize;
        let block_va = 0x1000u32; // page RVA
        let block_sz = 8u32 + 2; // header + 1 entry
        data[block_rva..block_rva + 4].copy_from_slice(&block_va.to_le_bytes());
        data[block_rva + 4..block_rva + 8].copy_from_slice(&block_sz.to_le_bytes());
        // 1 DIR64 entry: type=10, offset=0x42 -> entry=0xA042
        let entry: u16 = (10u16 << 12) | 0x042;
        data[block_rva + 8..block_rva + 10].copy_from_slice(&entry.to_le_bytes());

        data
    }

    #[test]
    fn parse_pe32_plus_ok() {
        let data = fake_pe32_plus(0x100, 10, 0x4000_0000);
        let pe = parse_pe32_plus(&data).unwrap();
        assert_eq!(pe.preferred_base, 0x4000_0000);
        assert_eq!(pe.reloc_rva, 0x100);
        assert_eq!(pe.reloc_size, 10);
    }

    #[test]
    fn apply_dir64_relocation() {
        let data = fake_pe32_plus(0x100, 10, 0x4000_0000);
        let mut bus = SystemBus::new();
        let load_base = 0x4008_0000;
        // Place original 64-bit value at load_base + 0x1000 + 0x42
        let target = load_base + 0x1000 + 0x42;
        bus.write(target, 8, 0x4000_DEAD_BEEF_CAFE);

        apply_pe_relocations(&mut bus, load_base, &data).unwrap();

        let fixed = bus.read(target, 8).unwrap();
        let expected = 0x4000_DEAD_BEEF_CAFEu64.wrapping_add(0x8_0000);
        assert_eq!(fixed, expected);
    }

    #[test]
    fn no_relocation_when_delta_zero() {
        let data = fake_pe32_plus(0x100, 10, 0x4008_0000);
        let mut bus = SystemBus::new();
        let load_base = 0x4008_0000;
        let target = load_base + 0x1000 + 0x42;
        bus.write(target, 8, 0xCAFE);

        apply_pe_relocations(&mut bus, load_base, &data).unwrap();

        assert_eq!(bus.read(target, 8).unwrap(), 0xCAFE);
    }

    #[test]
    fn no_relocation_when_reloc_size_zero() {
        let mut data = fake_pe32_plus(0x100, 0, 0x4000_0000);
        // Also zero out the rva to be safe
        let opt_start = 0x40 + 4 + 20;
        let dd5 = opt_start + 112 + 5 * 8;
        data[dd5..dd5 + 4].copy_from_slice(&0u32.to_le_bytes());
        data[dd5 + 4..dd5 + 8].copy_from_slice(&0u32.to_le_bytes());

        let mut bus = SystemBus::new();
        let target = 0x4008_0000 + 0x1000 + 0x42;
        bus.write(target, 8, 0xCAFE);

        apply_pe_relocations(&mut bus, 0x4008_0000, &data).unwrap();
        assert_eq!(bus.read(target, 8).unwrap(), 0xCAFE);
    }
}
