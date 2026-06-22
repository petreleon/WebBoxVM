use super::super::load::jit_load_guest_from_machine;
use super::super::store::stage_jit_store_from_machine;
use crate::constants::{RAM_BASE, SCTLR_MMU_ENABLE};
use crate::host::wasm::{Emulator, JitPendingExclusiveReservation, JitPendingStore};
use crate::runtime::Machine;

#[test]
fn jit_sync_clears_staged_side_effects() {
    let mut emulator = Emulator::new(Some(1));
    emulator.jit_helper_failed = true;
    emulator.jit_pending_exclusive_clear = Some(0);
    emulator.jit_pending_exclusive_reservation = Some(JitPendingExclusiveReservation {
        core_id: 0,
        pa: RAM_BASE,
        size: 8,
    });
    emulator
        .jit_pending_stores
        .push(JitPendingStore::new(RAM_BASE, &[1, 2, 3, 4]));

    assert!(emulator.jit_sync_state_from_core(Some(0)));

    assert!(!emulator.jit_helper_failed);
    assert!(emulator.jit_pending_exclusive_clear.is_none());
    assert!(emulator.jit_pending_exclusive_reservation.is_none());
    assert!(emulator.jit_pending_stores.is_empty());
}

#[test]
fn helper_failure_clears_staged_side_effects() {
    let mut emulator = Emulator::new(Some(1));
    emulator.jit_pending_exclusive_clear = Some(0);
    emulator.jit_pending_exclusive_reservation = Some(JitPendingExclusiveReservation {
        core_id: 0,
        pa: RAM_BASE,
        size: 8,
    });
    emulator
        .jit_pending_stores
        .push(JitPendingStore::new(RAM_BASE, &[1, 2, 3, 4]));

    assert_eq!(emulator.jit_load_guest(Some(0), RAM_BASE, 3), 0);

    assert!(emulator.jit_helper_failed);
    assert!(
        emulator
            .jit_last_error
            .contains("unsupported JIT load size 3")
    );
    assert!(emulator.jit_pending_exclusive_clear.is_none());
    assert!(emulator.jit_pending_exclusive_reservation.is_none());
    assert!(emulator.jit_pending_stores.is_empty());
}

#[test]
fn translation_fault_helpers_do_not_write_far_el1() {
    let mut machine = Machine::new(1);
    let mut stores = Vec::new();
    let cpu = &mut machine.cpus[0];
    cpu.sys.sctlr_el1 = SCTLR_MMU_ENABLE;
    cpu.sys.far_el1 = 0xfeed_face;

    let load_err = jit_load_guest_from_machine(&mut machine, 0, 0x4000, 8, &[])
        .expect_err("unmapped JIT load should fail");
    let store_err = stage_jit_store_from_machine(&mut machine, 0, 0x4000, 8, 1, &mut stores)
        .expect_err("unmapped JIT store should fail");

    assert!(load_err.contains("JIT load helper"));
    assert!(store_err.contains("JIT store helper"));
    assert_eq!(machine.cpus[0].sys.far_el1, 0xfeed_face);
    assert!(stores.is_empty());
}
