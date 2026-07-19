use super::super::pair_load::jit_load_pair_guest_from_machine;
use super::super::pair_store::stage_jit_pair_store_from_machine;
use super::super::store::{apply_jit_pending_stores, stage_jit_store_from_machine};
use crate::constants::{
    DESC_AF_BIT, DESC_BLOCK, DESC_TABLE, PAGE_SIZE, RAM_BASE, SCTLR_MMU_ENABLE, TCR_T1SZ_SHIFT,
};
use crate::runtime::Machine;

fn map_two_ttbr0_pages(machine: &mut Machine, page0_pa: u64, page1_pa: u64) {
    let l1_table = RAM_BASE;
    let l2_table = RAM_BASE + 0x1000;
    let l3_table = RAM_BASE + 0x2000;

    machine.bus.mem.write(l1_table, 8, l2_table | DESC_TABLE);
    machine.bus.mem.write(l2_table, 8, l3_table | DESC_TABLE);
    machine
        .bus
        .mem
        .write(l3_table, 8, page0_pa | DESC_AF_BIT | DESC_BLOCK);
    machine
        .bus
        .mem
        .write(l3_table + 8, 8, page1_pa | DESC_AF_BIT | DESC_BLOCK);

    let cpu = &mut machine.cpus[0];
    cpu.sys.ttbr0_el1 = l1_table;
    cpu.sys.tcr_el1 = (25 << TCR_T1SZ_SHIFT) | 25;
    cpu.sys.sctlr_el1 = SCTLR_MMU_ENABLE;
}

#[test]
fn jit_pair_store_stages_both_values_before_commit() {
    let mut machine = Machine::new(1);
    let mut stores = Vec::new();
    let base = RAM_BASE + 0x4000;

    stage_jit_pair_store_from_machine(
        &mut machine,
        0,
        base,
        8,
        0x1122_3344_5566_7788,
        0x99aa_bbcc_ddee_ff00,
        &mut stores,
    )
    .expect("pair store should stage RAM writes");

    assert_eq!(machine.bus.mem.read(base, 8), Some(0));
    assert_eq!(stores.len(), 1);
    apply_jit_pending_stores(&mut machine, 0, &stores).expect("apply staged pair stores");
    assert_eq!(machine.bus.mem.read(base, 8), Some(0x1122_3344_5566_7788));
    assert_eq!(
        machine.bus.mem.read(base + 8, 8),
        Some(0x99aa_bbcc_ddee_ff00)
    );
}

#[test]
fn jit_pair_store_keeps_staging_atomic_when_second_value_faults() {
    let mut machine = Machine::new(1);
    let mut stores = Vec::new();
    let page0_pa = RAM_BASE + 0x4000;
    let page1_pa = RAM_BASE + 0x8000;

    map_two_ttbr0_pages(&mut machine, page0_pa, page1_pa);
    machine.bus.mem.write(RAM_BASE + 0x2008, 8, 0);

    let err = stage_jit_pair_store_from_machine(
        &mut machine,
        0,
        0xff8,
        8,
        0x1122_3344_5566_7788,
        0x99aa_bbcc_ddee_ff00,
        &mut stores,
    )
    .expect_err("second pair value should fault");

    assert!(err.contains("JIT store helper"), "{err}");
    assert!(stores.is_empty());
}

#[test]
fn jit_pair_store_pending_span_forwards_before_commit() {
    let mut machine = Machine::new(1);
    let mut stores = Vec::new();
    let base = RAM_BASE + 0x5000;

    stage_jit_pair_store_from_machine(&mut machine, 0, base, 8, 0x11, 0x22, &mut stores)
        .expect("pair store should stage one span");

    let values = jit_load_pair_guest_from_machine(&mut machine, 0, base, 8, &stores)
        .expect("merged pending pair store should forward");

    assert_eq!(stores.len(), 1);
    assert_eq!(values, (0x11, 0x22));
    assert_eq!(machine.bus.mem.read(base, 8), Some(0));
}

#[test]
fn jit_pair_store_falls_back_across_noncontiguous_pages() {
    let mut machine = Machine::new(1);
    let mut stores = Vec::new();
    map_two_ttbr0_pages(&mut machine, RAM_BASE + 0x3000, RAM_BASE + 0x8000);

    stage_jit_pair_store_from_machine(
        &mut machine,
        0,
        PAGE_SIZE - 4,
        4,
        0x1122_3344,
        0x5566_7788,
        &mut stores,
    )
    .expect("cross-page pair store should translate both pages");

    assert_eq!(stores.len(), 2);
    apply_jit_pending_stores(&mut machine, 0, &stores).expect("apply staged pair stores");
    assert_eq!(
        machine.bus.mem.read(RAM_BASE + 0x3ffc, 4),
        Some(0x1122_3344)
    );
    assert_eq!(
        machine.bus.mem.read(RAM_BASE + 0x8000, 4),
        Some(0x5566_7788)
    );
}

#[test]
fn scalar_store_helper_still_supports_subword_stores() {
    let mut machine = Machine::new(1);
    let mut stores = Vec::new();
    let base = RAM_BASE + 0x9000;

    stage_jit_store_from_machine(&mut machine, 0, base, 2, 0xaabb, &mut stores)
        .expect("scalar helper keeps existing widths");

    apply_jit_pending_stores(&mut machine, 0, &stores).expect("apply staged store");
    assert_eq!(machine.bus.mem.read(base, 2), Some(0xaabb));
}
