use super::*;
use crate::bus::SystemBus;

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
