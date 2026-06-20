use super::*;

#[test]
fn ttbr1_maps_kernel_va_to_kernel_load_pa() {
    let mut bus = SystemBus::new();
    let mut sys = SystemRegisters::default();
    let l1_block = |pa: u64| -> u64 { pa | DESC_AF_BIT | DESC_BLOCK };
    let l3_page = |pa: u64| -> u64 { pa | DESC_AF_BIT | DESC_VALID };

    bus.write(
        BOOT_TTBR0_L0,
        8,
        (BOOT_TTBR0_L1 & DESC_ADDR_MASK) | DESC_VALID,
    );
    for i in 0..IDENTITY_MAP_BLOCKS {
        bus.write(
            BOOT_TTBR0_L1 + i as u64 * 8,
            8,
            l1_block(i as u64 * L1_BLOCK_SIZE),
        );
    }

    bus.write(
        BOOT_TTBR1_L0 + 256 * 8,
        8,
        (BOOT_TTBR1_L1 & DESC_ADDR_MASK) | DESC_VALID,
    );
    bus.write(
        BOOT_TTBR1_L1,
        8,
        (BOOT_TTBR1_L2 & DESC_ADDR_MASK) | DESC_VALID,
    );
    bus.write(
        BOOT_TTBR1_L1 + 2 * 8,
        8,
        (BOOT_TTBR1_L2 & DESC_ADDR_MASK) | DESC_VALID,
    );

    for tbl in 0..BOOT_TTBR1_L3_COUNT {
        let l3_table_addr = BOOT_TTBR1_L3_BASE + (tbl as u64) * PAGE_SIZE;
        bus.write(
            BOOT_TTBR1_L2 + (tbl as u64) * 8,
            8,
            (l3_table_addr & DESC_ADDR_MASK) | DESC_VALID,
        );
        for i in 0..PT_ENTRIES as usize {
            let va_offset = (tbl as u64) * L2_BLOCK_SIZE + (i as u64) * PAGE_SIZE;
            bus.write(
                l3_table_addr + i as u64 * 8,
                8,
                l3_page(KERNEL_LOAD_ADDR + 0x10000 + va_offset),
            );
        }
    }

    sys.ttbr0_el1 = BOOT_TTBR0_L0;
    sys.ttbr1_el1 = BOOT_TTBR1_L0;
    sys.tcr_el1 = (16 << TCR_T1SZ_SHIFT) | 16;
    sys.mair_el1 = MAIR_EL1_DEFAULT;
    sys.sctlr_el1 = SCTLR_MMU_ENABLE;

    let test_va = KERNEL_VA_BASE;
    let expected_pa = KERNEL_LOAD_ADDR + 0x10000;
    let mut tlb = Tlb::new();
    let pa = translate(&sys, &mut tlb, &bus.mem, test_va).expect("translation should succeed");

    assert_eq!(
        pa, expected_pa,
        "VA 0x{test_va:016x} should map to KERNEL_LOAD + .text_RVA = 0x{expected_pa:016x}, got 0x{pa:016x}"
    );
}
