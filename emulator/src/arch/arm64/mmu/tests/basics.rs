use super::*;

#[test]
fn mmu_off_passes_through() {
    let sys = SystemRegisters::default();
    let mut tlb = Tlb::new();
    let mem = PhysicalMemory::new();
    assert_eq!(
        translate(&sys, &mut tlb, &mem, 0x4000_0000).unwrap(),
        0x4000_0000
    );
}

#[test]
fn tlb_hit_caches_translation() {
    let (bus, sys) = mapped_page_fixture(0x4000_3000);
    let mut tlb = Tlb::new();

    assert_eq!(
        translate(&sys, &mut tlb, &bus.mem, 0xFFFF_FF80_0000_0000).unwrap(),
        0x4000_3000
    );
    assert_eq!(
        translate(&sys, &mut tlb, &bus.mem, 0xFFFF_FF80_0000_0001).unwrap(),
        0x4000_3001
    );
}

#[test]
fn page_table_walk_4kb_page() {
    let mut bus = SystemBus::new();
    let mut sys = SystemRegisters::default();
    let l1_table = 0x4000_0000;
    let l2_table = 0x4000_1000;
    let l3_table = 0x4000_2000;

    bus.mem.write(l1_table, 8, (l2_table | 0b11) as u64);
    bus.mem.write(l2_table, 8, (l3_table | 0b11) as u64);
    bus.mem.write(l3_table, 8, (0x4000_3000u64 | 0b01) as u64);

    sys.ttbr1_el1 = l1_table;
    sys.tcr_el1 = (25 << 16) | 25;
    sys.sctlr_el1 = 1;

    let mut tlb = Tlb::new();
    let pa = translate(&sys, &mut tlb, &bus.mem, 0xFFFF_FF80_0000_0000).unwrap();
    assert_eq!(pa, 0x4000_3000);
}

#[test]
fn page_table_walk_2mb_block() {
    let mut bus = SystemBus::new();
    let mut sys = SystemRegisters::default();
    let l1_table = 0x4000_0000;
    let l2_table = 0x4000_1000;

    bus.mem.write(l1_table, 8, (l2_table | 0b11) as u64);
    bus.mem.write(l2_table, 8, (0x4000_0000u64 | 0b01) as u64);

    sys.ttbr1_el1 = l1_table;
    sys.tcr_el1 = (25 << 16) | 25;
    sys.sctlr_el1 = 1;

    let mut tlb = Tlb::new();
    let pa = translate(&sys, &mut tlb, &bus.mem, 0xFFFF_FF80_0000_1000).unwrap();
    assert_eq!(pa, 0x4000_1000);
}

#[test]
fn page_table_walk_1gb_block() {
    let mut bus = SystemBus::new();
    let mut sys = SystemRegisters::default();
    let l1_table = 0x4000_0000;

    bus.mem.write(l1_table, 8, (0x4000_0000u64 | 0b01) as u64);

    sys.ttbr1_el1 = l1_table;
    sys.tcr_el1 = (25 << 16) | 25;
    sys.sctlr_el1 = 1;

    let mut tlb = Tlb::new();
    let pa = translate(&sys, &mut tlb, &bus.mem, 0xFFFF_FF80_0000_1000).unwrap();
    assert_eq!(pa, 0x4000_1000);
}

#[test]
fn invalid_descriptor_faults() {
    let mut bus = SystemBus::new();
    let mut sys = SystemRegisters::default();
    let l1_table = 0x4000_0000;
    bus.mem.write(l1_table, 8, 0);

    sys.ttbr1_el1 = l1_table;
    sys.tcr_el1 = (25 << 16) | 25;
    sys.sctlr_el1 = 1;

    let mut tlb = Tlb::new();
    assert!(translate(&sys, &mut tlb, &bus.mem, 0xFFFF_FF80_0000_0000).is_err());
}

#[test]
fn tlbi_invalidates_tlb() {
    let (bus, sys) = mapped_page_fixture(0x4000_3000);
    let mut tlb = Tlb::new();

    let _ = translate(&sys, &mut tlb, &bus.mem, 0xFFFF_FF80_0000_0000).unwrap();
    assert!(tlb.entries.iter().any(|entry| entry.valid));

    tlb.invalidate_all();
    assert!(tlb.entries.iter().all(|entry| !entry.valid));
}

#[test]
fn descriptor_generation_change_invalidates_translation() {
    let (mut bus, sys) = mapped_page_fixture(0x4000_3000);
    let mut tlb = Tlb::new();
    let va = 0xFFFF_FF80_0000_0000;

    assert_eq!(
        translate(&sys, &mut tlb, &bus.mem, va).unwrap(),
        0x4000_3000
    );
    bus.mem.write(0x4000_2000, 8, 0x4000_5000u64 | 0b01);

    assert_eq!(
        translate(&sys, &mut tlb, &bus.mem, va).unwrap(),
        0x4000_5000
    );
}

#[test]
fn context_change_invalidates_translation() {
    let (mut bus, mut sys) = mapped_page_fixture(0x4000_3000);
    let alt_l1 = 0x4000_6000;
    let alt_l2 = 0x4000_7000;
    let alt_l3 = 0x4000_8000;
    let va = 0xFFFF_FF80_0000_0000;
    let mut tlb = Tlb::new();

    bus.mem.write(alt_l1, 8, alt_l2 | 0b11);
    bus.mem.write(alt_l2, 8, alt_l3 | 0b11);
    bus.mem.write(alt_l3, 8, 0x4000_9000u64 | 0b01);

    assert_eq!(
        translate(&sys, &mut tlb, &bus.mem, va).unwrap(),
        0x4000_3000
    );
    sys.ttbr1_el1 = alt_l1;

    assert_eq!(
        translate(&sys, &mut tlb, &bus.mem, va).unwrap(),
        0x4000_9000
    );
}

fn mapped_page_fixture(pa: u64) -> (SystemBus, SystemRegisters) {
    let mut bus = SystemBus::new();
    let mut sys = SystemRegisters::default();
    let l1_table = 0x4000_0000;
    let l2_table = 0x4000_1000;
    let l3_table = 0x4000_2000;

    bus.mem.write(l1_table, 8, l2_table | 0b11);
    bus.mem.write(l2_table, 8, l3_table | 0b11);
    bus.mem.write(l3_table, 8, pa | 0b01);
    sys.ttbr1_el1 = l1_table;
    sys.tcr_el1 = (25 << TCR_T1SZ_SHIFT) | 25;
    sys.sctlr_el1 = SCTLR_MMU_ENABLE;
    (bus, sys)
}
