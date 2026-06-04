use super::*;

const EFI_RELOCATION_LOOP_ENTRY: u64 = 0x4196_b450;
const EFI_RELOCATION_LOOP_EXIT: u64 = 0x4196_b474;
const R_AARCH64_RELATIVE: u64 = 0x403;

pub(super) fn setup_boot_page_tables(cpu: &mut Armv8Cpu, bus: &mut SystemBus) {
    let ttbr1_l0 = 0x8010_0000u64;
    let ttbr1_l1 = 0x8010_1000u64;
    let ttbr1_l2 = 0x8010_2000u64;
    let ttbr1_l3_base = 0x8010_3000u64;
    let num_l3_tables = 96usize;
    let ttbr0_l0 = 0x8017_3000u64;
    let ttbr0_l1 = 0x8017_4000u64;

    let l1_block = |pa: u64| -> u64 { pa | (1 << 10) | 0b01 };
    let l3_page = |pa: u64| -> u64 { pa | (1 << 10) | 0b11 };

    bus.write(ttbr0_l0, 8, (ttbr0_l1 & 0x0000_FFFF_FFFF_F000) | 0b11);
    for i in 0..4 {
        bus.write(ttbr0_l1 + i * 8, 8, l1_block(i * 0x4000_0000));
    }

    bus.write(
        ttbr1_l0 + 256 * 8,
        8,
        (ttbr1_l1 & 0x0000_FFFF_FFFF_F000) | 0b11,
    );
    bus.write(
        ttbr1_l1 + 2 * 8,
        8,
        (ttbr1_l2 & 0x0000_FFFF_FFFF_F000) | 0b11,
    );

    for tbl in 0..num_l3_tables {
        let l3 = ttbr1_l3_base + (tbl as u64) * 0x1000;
        bus.write(
            ttbr1_l2 + (tbl as u64) * 8,
            8,
            (l3 & 0x0000_FFFF_FFFF_F000) | 0b11,
        );
        for i in 0..512 {
            let va_offset = (tbl as u64) * 0x20_0000 + (i as u64) * 0x1000;
            bus.write(l3 + i * 8, 8, l3_page(0x4800_0000 + va_offset));
        }
    }

    cpu.sys.ttbr0_el1 = ttbr0_l0;
    cpu.sys.ttbr1_el1 = ttbr1_l0;
    cpu.sys.tcr_el1 = (16 << 16) | 16;
    cpu.sys.mair_el1 = 0xFF;
    cpu.sys.sctlr_el1 = 1;
}

pub(super) fn fast_forward_efi_relocation_loop(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
) -> Option<usize> {
    if cpu.regs.pc != EFI_RELOCATION_LOOP_ENTRY {
        return None;
    }

    let mut record = cpu.regs.x(9);
    let end = cpu.regs.x(10);
    let delta = cpu.regs.x(23);
    let mut records = 0usize;

    while record < end {
        let target = bus.mem.read(record, 8).unwrap_or(0);
        let reloc_type = bus.mem.read(record + 8, 8).unwrap_or(0);
        let addend = bus.mem.read(record + 16, 8).unwrap_or(0);

        if (reloc_type & 0xffff_ffff) == R_AARCH64_RELATIVE {
            bus.mem
                .write(target.wrapping_add(delta), 8, addend.wrapping_add(delta));
        }

        record = record.wrapping_add(24);
        records += 1;
    }

    cpu.regs.set_x(9, record);
    cpu.regs.pc = EFI_RELOCATION_LOOP_EXIT;
    Some(records)
}
