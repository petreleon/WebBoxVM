use super::*;

#[test]
fn read_only_translation_uses_cache_without_filling_tlb() {
    let mut bus = SystemBus::new();
    let mut sys = SystemRegisters::default();
    let va = 0xFFFF_FF80_0000_0000;
    let l1 = 0x4000_0000;
    let l2 = 0x4000_1000;
    let l3 = 0x4000_2000;
    bus.mem.write(l1, 8, l2 | 0b11);
    bus.mem.write(l2, 8, l3 | 0b11);
    bus.mem.write(l3, 8, 0x4000_3000u64 | 0b01);
    sys.ttbr1_el1 = l1;
    sys.tcr_el1 = (25 << TCR_T1SZ_SHIFT) | 25;
    sys.sctlr_el1 = SCTLR_MMU_ENABLE;

    let mut cached = Tlb::new();
    let empty = Tlb::new();
    translate(&sys, &mut cached, &bus.mem, va).unwrap();
    assert_eq!(
        translate_read_only(&sys, Some(&cached), &bus.mem, va + 1).unwrap(),
        0x4000_3001
    );
    assert_eq!(
        translate_read_only(&sys, Some(&empty), &bus.mem, va).unwrap(),
        0x4000_3000
    );
    assert!(empty.entries.iter().all(|entry| !entry.valid));
}
