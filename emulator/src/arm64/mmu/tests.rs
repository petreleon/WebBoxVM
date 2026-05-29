use super::*;
use crate::bus::SystemBus;
use crate::constants::*;

#[test]
fn mmu_off_passes_through() {
    let sys = SystemRegisters::default();
    let mut tlb = Tlb::new();
    let mem = PhysicalMemory::new();
    assert_eq!(translate(&sys, &mut tlb, &mem, 0x4000_0000).unwrap(), 0x4000_0000);
}

#[test]
fn tlb_hit_caches_translation() {
    let mut tlb = Tlb::new();
    tlb.insert(0xFFFF_FF80_0000_0000, 0x4000_0000);
    let sys = SystemRegisters { sctlr_el1: 1, ..SystemRegisters::default() };
    let mem = PhysicalMemory::new();
    assert_eq!(translate(&sys, &mut tlb, &mem, 0xFFFF_FF80_0000_0000).unwrap(), 0x4000_0000);
    assert_eq!(translate(&sys, &mut tlb, &mem, 0xFFFF_FF80_0000_0001).unwrap(), 0x4000_0001);
}

#[test]
fn page_table_walk_4kb_page() {
    let mut bus = SystemBus::new();
    let mut sys = SystemRegisters::default();

    // Build page tables at PA 0x4000_0000
    let l1_table = 0x4000_0000;
    let l2_table = 0x4000_1000;
    let l3_table = 0x4000_2000;

    // L1 descriptor: table pointer to L2
    bus.mem.write(l1_table, 8, (l2_table | 0b11) as u64);

    // L2 descriptor: table pointer to L3
    bus.mem.write(l2_table, 8, (l3_table | 0b11) as u64);

    // L3 descriptor: 4KB page at PA 0x4000_3000
    bus.mem.write(l3_table, 8, (0x4000_3000u64 | 0b01) as u64);

    sys.ttbr1_el1 = l1_table;
    sys.tcr_el1 = (25 << 16) | 25; // T1SZ=25, T0SZ=25 (39-bit)
    sys.sctlr_el1 = 1;

    let mut tlb = Tlb::new();
    let va = 0xFFFF_FF80_0000_0000;
    let pa = translate(&sys, &mut tlb, &bus.mem, va).unwrap();
    assert_eq!(pa, 0x4000_3000);
}

#[test]
fn page_table_walk_2mb_block() {
    let mut bus = SystemBus::new();
    let mut sys = SystemRegisters::default();

    let l1_table = 0x4000_0000;
    let l2_table = 0x4000_1000;

    bus.mem.write(l1_table, 8, (l2_table | 0b11) as u64);
    // L2 block descriptor: 2MB block at PA 0x4000_0000
    bus.mem.write(l2_table, 8, (0x4000_0000u64 | 0b01) as u64);

    sys.ttbr1_el1 = l1_table;
    sys.tcr_el1 = (25 << 16) | 25;
    sys.sctlr_el1 = 1;

    let mut tlb = Tlb::new();
    let va = 0xFFFF_FF80_0000_1000;
    let pa = translate(&sys, &mut tlb, &bus.mem, va).unwrap();
    assert_eq!(pa, 0x4000_1000);
}

#[test]
fn page_table_walk_1gb_block() {
    let mut bus = SystemBus::new();
    let mut sys = SystemRegisters::default();

    let l1_table = 0x4000_0000;
    // L1 block descriptor: 1GB block at PA 0x4000_0000
    bus.mem.write(l1_table, 8, (0x4000_0000u64 | 0b01) as u64);

    sys.ttbr1_el1 = l1_table;
    sys.tcr_el1 = (25 << 16) | 25;
    sys.sctlr_el1 = 1;

    let mut tlb = Tlb::new();
    let va = 0xFFFF_FF80_0000_1000;
    let pa = translate(&sys, &mut tlb, &bus.mem, va).unwrap();
    assert_eq!(pa, 0x4000_1000);
}

#[test]
fn invalid_descriptor_faults() {
    let mut bus = SystemBus::new();
    let mut sys = SystemRegisters::default();

    let l1_table = 0x4000_0000;
    bus.mem.write(l1_table, 8, 0); // invalid

    sys.ttbr1_el1 = l1_table;
    sys.tcr_el1 = (25 << 16) | 25;
    sys.sctlr_el1 = 1;

    let mut tlb = Tlb::new();
    let va = 0xFFFF_FF80_0000_0000;
    assert!(translate(&sys, &mut tlb, &bus.mem, va).is_err());
}

#[test]
fn tlbi_invalidates_tlb() {
    let mut tlb = Tlb::new();
    tlb.insert(0xFFFF_FF80_0000_0000, 0x4000_0000);
    assert!(tlb.lookup(0xFFFF_FF80_0000_0000).is_some());
    tlb.invalidate_all();
    assert!(tlb.lookup(0xFFFF_FF80_0000_0000).is_none());
}

/// Test that TTBR1 maps kernel VAs to the correct physical address where
/// the kernel code is actually loaded (KERNEL_LOAD_ADDR, not RAM_BASE).
/// This is a regression test for the mapping bug that causes the kernel
/// to execute wrong code after MMU enable.
///
/// The kernel's .text section starts at RVA 0x10000, so the first real
/// instruction (at VA PAGE_OFFSET, the _text symbol) should map to
/// KERNEL_LOAD + 0x10000, not KERNEL_LOAD + 0.
#[test]
fn ttbr1_maps_kernel_va_to_kernel_load_pa() {
    let mut bus = SystemBus::new();
    let mut sys = SystemRegisters::default();

    // Replicate setup_boot_page_tables from boot.rs — same L0/L1/L2/L3 layout
    let l1_block = |pa: u64| -> u64 { pa | DESC_AF_BIT | DESC_BLOCK };
    let l3_page  = |pa: u64| -> u64 { pa | DESC_AF_BIT | DESC_VALID };

    // TTBR0: identity map first 4 GB
    bus.write(BOOT_TTBR0_L0, 8, (BOOT_TTBR0_L1 & DESC_ADDR_MASK) | DESC_VALID);
    for i in 0..IDENTITY_MAP_BLOCKS {
        bus.write(BOOT_TTBR0_L1 + i as u64 * 8, 8, l1_block(i as u64 * L1_BLOCK_SIZE));
    }

    // TTBR1: map kernel VA → KERNEL_LOAD_ADDR via L3 pages
    bus.write(BOOT_TTBR1_L0 + 256 * 8, 8, (BOOT_TTBR1_L1 & DESC_ADDR_MASK) | DESC_VALID);
    bus.write(BOOT_TTBR1_L1 + 0 * 8, 8, (BOOT_TTBR1_L2 & DESC_ADDR_MASK) | DESC_VALID);
    bus.write(BOOT_TTBR1_L1 + 2 * 8, 8, (BOOT_TTBR1_L2 & DESC_ADDR_MASK) | DESC_VALID);

    for tbl in 0..BOOT_TTBR1_L3_COUNT {
        let l3_table_addr = BOOT_TTBR1_L3_BASE + (tbl as u64) * PAGE_SIZE;
        bus.write(BOOT_TTBR1_L2 + (tbl as u64) * 8, 8, (l3_table_addr & DESC_ADDR_MASK) | DESC_VALID);
        for i in 0..PT_ENTRIES as usize {
            let va_offset = (tbl as u64) * L2_BLOCK_SIZE + (i as u64) * PAGE_SIZE;
            bus.write(l3_table_addr + i as u64 * 8, 8, l3_page(KERNEL_LOAD_ADDR + 0x10000 + va_offset));
        }
    }

    sys.ttbr0_el1 = BOOT_TTBR0_L0;
    sys.ttbr1_el1 = BOOT_TTBR1_L0;
    sys.tcr_el1 = (16 << TCR_T1SZ_SHIFT) | 16;
    sys.mair_el1 = MAIR_EL1_DEFAULT;
    sys.sctlr_el1 = SCTLR_MMU_ENABLE;

    // The kernel's _text symbol is at PAGE_OFFSET + text_offset (= KERNEL_VA_BASE + 0).
    // The actual code for _text lives in the .text section at file offset 0x10000.
    // So VA KERNEL_VA_BASE should map to PA KERNEL_LOAD_ADDR + 0x10000.
    // (This is the bug: currently we map to KERNEL_LOAD_ADDR + 0, not + 0x10000)
    let test_va: u64 = KERNEL_VA_BASE;
    let text_rva: u64 = 0x10000; // .text section RVA from PE header
    let expected_pa: u64 = KERNEL_LOAD_ADDR + text_rva;

    let mut tlb = Tlb::new();
    let pa = translate(&sys, &mut tlb, &bus.mem, test_va)
        .expect("translation should succeed");

    assert_eq!(pa, expected_pa,
        "VA 0x{:016x} (_text) should map to KERNEL_LOAD + .text_RVA = 0x{:016x}, got 0x{:016x}\n\
         (The kernel's .text section starts at file offset 0x10000, not 0x0)",
        test_va, expected_pa, pa);
}
