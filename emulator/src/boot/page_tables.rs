use crate::arm64::Armv8Cpu;
use crate::bus::SystemBus;
use crate::constants::*;

pub(super) fn setup_boot_page_tables(cpu: &mut Armv8Cpu, bus: &mut SystemBus) {
    // Helper: encode an L1 block descriptor (1 GiB) — bit 10=AF, [1:0]=01=block
    let l1_block = |pa: u64| -> u64 { pa | DESC_AF_BIT | DESC_BLOCK };
    // Helper: encode an L3 page descriptor (4 KiB) — bit 10=AF, [1:0]=11=page
    let l3_page  = |pa: u64| -> u64 { pa | DESC_AF_BIT | DESC_VALID };

    // ── TTBR0: identity-map the first 4 GiB ──
    // 4 × 1 GiB blocks cover the entire low + RAM regions
    bus.write(BOOT_TTBR0_L0, 8, (BOOT_TTBR0_L1 & DESC_ADDR_MASK) | DESC_VALID);
    for i in 0..IDENTITY_MAP_BLOCKS {
        bus.write(BOOT_TTBR0_L1 + i as u64 * 8, 8, l1_block(i as u64 * L1_BLOCK_SIZE));
    }

    // ── TTBR1: map kernel VA → physical PA ──
    // L0 entry at index 256 (= kernel-space starts at VA bit 47)
    bus.write(BOOT_TTBR1_L0 + 256 * 8, 8, (BOOT_TTBR1_L1 & DESC_ADDR_MASK) | DESC_VALID);
    // L1 entries at index 0 and 2 — cover different kernel VA layouts
    bus.write(BOOT_TTBR1_L1 + 0 * 8, 8, (BOOT_TTBR1_L2 & DESC_ADDR_MASK) | DESC_VALID);
    bus.write(BOOT_TTBR1_L1 + 2 * 8, 8, (BOOT_TTBR1_L2 & DESC_ADDR_MASK) | DESC_VALID);
    // L2 → L3 for each 2 MiB region
    for tbl in 0..BOOT_TTBR1_L3_COUNT {
        let l3_table_addr = BOOT_TTBR1_L3_BASE + (tbl as u64) * PAGE_SIZE;
        bus.write(BOOT_TTBR1_L2 + (tbl as u64) * 8, 8, (l3_table_addr & DESC_ADDR_MASK) | DESC_VALID);
        // Fill L3 with 4 KiB page entries
        for i in 0..PT_ENTRIES as usize {
            let va_offset = (tbl as u64) * L2_BLOCK_SIZE + (i as u64) * PAGE_SIZE;
            bus.write(l3_table_addr + i as u64 * 8, 8, l3_page(KERNEL_LOAD_ADDR + 0x10000 + va_offset));
        }
    }

    // Configure MMU registers
    cpu.sys.ttbr0_el1 = BOOT_TTBR0_L0;
    cpu.sys.ttbr1_el1 = BOOT_TTBR1_L0;
    cpu.sys.tcr_el1 = (16 << TCR_T1SZ_SHIFT) | 16; // 48-bit VA space
    cpu.sys.mair_el1 = MAIR_EL1_DEFAULT;
    cpu.sys.sctlr_el1 = SCTLR_MMU_ENABLE; // enable the MMU
}
