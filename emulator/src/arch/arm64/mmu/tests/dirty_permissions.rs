use super::*;

#[test]
fn dbm_write_clears_read_only_permission_when_hd_enabled() {
    let (mut bus, mut sys, desc_addr, clean_writable) = dirty_page_fixture();
    sys.tcr_el1 = TCR_HA_BIT | TCR_HD_BIT | (25 << TCR_T1SZ_SHIFT) | 25;
    sys.sctlr_el1 = SCTLR_MMU_ENABLE;

    let mut tlb = Tlb::new();
    let pa = translate_write(&sys, &mut tlb, &mut bus.mem, 0x1234, 0).unwrap();
    let updated = bus.mem.read(desc_addr, 8).unwrap();

    assert_eq!(pa, 0x4000_3000 + (0x1234 & PAGE_OFFSET_MASK));
    assert_eq!(updated & DESC_AP_RO, 0);
    assert_eq!(updated & DESC_DBM_BIT, DESC_DBM_BIT);
    assert_eq!(updated & DESC_AP_EL0, DESC_AP_EL0);
    assert_eq!(clean_writable & DESC_AP_RO, DESC_AP_RO);
}

#[test]
fn dbm_write_faults_when_hd_is_disabled() {
    let (mut bus, mut sys, _, _) = dirty_page_fixture();
    sys.tcr_el1 = TCR_HA_BIT | (25 << TCR_T1SZ_SHIFT) | 25;
    sys.sctlr_el1 = SCTLR_MMU_ENABLE;
    let mut tlb = Tlb::new();

    assert_eq!(
        translate_write(&sys, &mut tlb, &mut bus.mem, 0x1234, 0),
        Err(Fault::PermissionFault)
    );
}

#[test]
fn linux_dirty_user_pte_still_faults_without_dbm() {
    let mut bus = SystemBus::new();
    let mut sys = SystemRegisters::default();
    let l1_table = 0x4000_0000;
    let l2_table = 0x4000_1000;
    let l3_table = 0x4000_2000;
    let desc_addr = l3_table + 8;
    let dirty_read_only =
        0x4000_3000 | DESC_VALID | DESC_AF_BIT | DESC_AP_EL0 | DESC_AP_RO | DESC_SW_DIRTY_BIT;

    bus.mem
        .write(l1_table, 8, (l2_table & DESC_ADDR_MASK) | DESC_VALID);
    bus.mem
        .write(l2_table, 8, (l3_table & DESC_ADDR_MASK) | DESC_VALID);
    bus.mem.write(desc_addr, 8, dirty_read_only);

    sys.ttbr0_el1 = l1_table;
    sys.tcr_el1 = (25 << TCR_T1SZ_SHIFT) | 25;
    sys.sctlr_el1 = SCTLR_MMU_ENABLE;
    let mut tlb = Tlb::new();

    assert_eq!(
        translate_write(&sys, &mut tlb, &mut bus.mem, 0x1234, 0),
        Err(Fault::PermissionFault)
    );
    assert_eq!(bus.mem.read(desc_addr, 8), Some(dirty_read_only));
}

#[test]
fn write_tlb_rechecks_when_descriptor_page_changes() {
    let (mut bus, mut sys, desc_addr, _) = dirty_page_fixture();
    sys.tcr_el1 = TCR_HA_BIT | TCR_HD_BIT | (25 << TCR_T1SZ_SHIFT) | 25;
    sys.sctlr_el1 = SCTLR_MMU_ENABLE;
    let mut tlb = Tlb::new();

    assert_eq!(
        translate_write(&sys, &mut tlb, &mut bus.mem, 0x1234, 0).unwrap(),
        0x4000_3000 + (0x1234 & PAGE_OFFSET_MASK)
    );
    bus.mem.write(desc_addr, 8, 0);

    assert_eq!(
        translate_write(&sys, &mut tlb, &mut bus.mem, 0x1235, 0),
        Err(Fault::TranslationFault)
    );
}

#[test]
fn privileged_write_tlb_entry_does_not_grant_el0_access() {
    let mut bus = SystemBus::new();
    let mut sys = SystemRegisters::default();
    let l1_table = 0x4000_0000;
    let l2_table = 0x4000_1000;
    let l3_table = 0x4000_2000;
    let privileged_rw = 0x4000_3000 | DESC_VALID | DESC_AF_BIT;

    bus.mem
        .write(l1_table, 8, (l2_table & DESC_ADDR_MASK) | DESC_VALID);
    bus.mem
        .write(l2_table, 8, (l3_table & DESC_ADDR_MASK) | DESC_VALID);
    bus.mem.write(l3_table + 8, 8, privileged_rw);
    sys.ttbr0_el1 = l1_table;
    sys.tcr_el1 = (25 << TCR_T1SZ_SHIFT) | 25;
    sys.sctlr_el1 = SCTLR_MMU_ENABLE;
    let mut tlb = Tlb::new();

    assert!(translate_write(&sys, &mut tlb, &mut bus.mem, 0x1234, 1).is_ok());
    assert_eq!(
        translate_write(&sys, &mut tlb, &mut bus.mem, 0x1234, 0),
        Err(Fault::PermissionFault)
    );
}

fn dirty_page_fixture() -> (SystemBus, SystemRegisters, u64, u64) {
    let mut bus = SystemBus::new();
    let mut sys = SystemRegisters::default();
    let l1_table = 0x4000_0000;
    let l2_table = 0x4000_1000;
    let l3_table = 0x4000_2000;
    let desc_addr = l3_table + 8;
    let clean_writable =
        0x4000_3000 | DESC_VALID | DESC_AF_BIT | DESC_AP_EL0 | DESC_AP_RO | DESC_DBM_BIT;

    bus.mem
        .write(l1_table, 8, (l2_table & DESC_ADDR_MASK) | DESC_VALID);
    bus.mem
        .write(l2_table, 8, (l3_table & DESC_ADDR_MASK) | DESC_VALID);
    bus.mem.write(desc_addr, 8, clean_writable);
    sys.ttbr0_el1 = l1_table;
    (bus, sys, desc_addr, clean_writable)
}
