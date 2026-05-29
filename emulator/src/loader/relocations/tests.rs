use super::*;
use crate::bus::SystemBus;

fn fake_pe32_plus(reloc_rva: u32, reloc_size: u32, preferred_base: u64) -> Vec<u8> {
    let mut data = vec![0u8; 0x200];
    // ARM64 kernel magic
    data[56..60].copy_from_slice(&ARM64_KERNEL_MAGIC.to_le_bytes());
    // e_lfanew at 0x3C
    data[0x3C..0x40].copy_from_slice(&0x40u32.to_le_bytes());
    // PE signature
    data[0x40..0x44].copy_from_slice(b"PE\0\0");
    // COFF header: 20 bytes
    let opt_start = 0x40 + 4 + 20;

    // Optional header magic
    data[opt_start..opt_start + 2].copy_from_slice(&PE32PLUS_MAGIC.to_le_bytes());
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
