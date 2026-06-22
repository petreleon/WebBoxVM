use super::super::exclusive::{
    apply_jit_pending_exclusive_clear, jit_store_exclusive_pair_from_machine,
};
use super::super::store::apply_jit_pending_stores;
use crate::constants::RAM_BASE;
use crate::runtime::Machine;

#[test]
fn jit_store_exclusive_pair_stages_successful_pair() {
    let mut machine = Machine::new(1);
    let mut stores = Vec::new();
    let addr = RAM_BASE + 0x200;
    machine.bus.mem.write(addr, 8, 0);
    machine.bus.mem.write(addr + 8, 8, 0);
    machine.cpus[0].reserve_exclusive(addr, 16);

    let status = jit_store_exclusive_pair_from_machine(
        &mut machine,
        0,
        addr,
        8,
        0x1122_3344_5566_7788,
        0x99aa_bbcc_ddee_ff00,
        &mut stores,
    )
    .expect("exclusive pair helper should succeed");

    assert_eq!(status, 0);
    assert_eq!(machine.bus.mem.read(addr, 8), Some(0));
    assert!(machine.cpus[0].exclusive_matches(addr, 16));
    apply_jit_pending_stores(&mut machine, &stores).expect("apply staged pair stores");
    apply_jit_pending_exclusive_clear(&mut machine, Some(0));
    assert_eq!(machine.bus.mem.read(addr, 8), Some(0x1122_3344_5566_7788));
    assert_eq!(
        machine.bus.mem.read(addr + 8, 8),
        Some(0x99aa_bbcc_ddee_ff00)
    );
    assert!(machine.cpus[0].exclusive.is_none());
}

#[test]
fn jit_store_exclusive_pair_reports_failed_reservation_without_stores() {
    let mut machine = Machine::new(1);
    let mut stores = Vec::new();
    let addr = RAM_BASE + 0x300;
    machine.bus.mem.write(addr, 8, 0);
    machine.bus.mem.write(addr + 8, 8, 0);
    machine.cpus[0].reserve_exclusive(addr + 0x40, 16);

    let status = jit_store_exclusive_pair_from_machine(&mut machine, 0, addr, 8, 1, 2, &mut stores)
        .expect("failed reservation should still be a valid STXP result");

    assert_eq!(status, 1);
    assert!(stores.is_empty());
    assert!(machine.cpus[0].exclusive_matches(addr + 0x40, 16));
    apply_jit_pending_exclusive_clear(&mut machine, Some(0));
    assert!(machine.cpus[0].exclusive.is_none());
}
