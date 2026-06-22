use super::super::exclusive::{
    apply_jit_pending_exclusive_clear, jit_store_exclusive_pair_from_machine,
};
use super::super::exclusive_load::{
    apply_jit_pending_exclusive_reservation, jit_load_exclusive_from_machine,
};
use super::super::store::apply_jit_pending_stores;
use crate::constants::{RAM_BASE, UART_BASE};
use crate::host::wasm::{Emulator, JitPendingExclusiveReservation};
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

#[test]
fn jit_load_exclusive_stages_reservation_until_commit() {
    let mut machine = Machine::new(1);
    let addr = RAM_BASE + 0x400;
    machine.bus.mem.write(addr, 4, 0x4433_2211);

    let (value, reservation) = jit_load_exclusive_from_machine(&mut machine, 0, addr, 4, &[])
        .expect("exclusive load helper should read RAM");

    assert_eq!(value, 0x4433_2211);
    assert!(machine.cpus[0].exclusive.is_none());
    apply_jit_pending_exclusive_reservation(&mut machine, Some(reservation));
    assert!(machine.cpus[0].exclusive_matches(addr, 4));
}

#[test]
fn jit_commit_applies_pending_exclusive_reservation() {
    let mut emulator = Emulator::new(Some(1));
    let addr = RAM_BASE + 0x500;
    emulator.jit_state.copy_from_cpu(&emulator.machine.cpus[0]);
    emulator.jit_pending_exclusive_reservation = Some(JitPendingExclusiveReservation {
        core_id: 0,
        pa: addr,
        size: 8,
    });

    assert!(emulator.jit_commit_state_to_core(Some(0), 1, 0));

    assert!(emulator.machine.cpus[0].exclusive_matches(addr, 8));
}

#[test]
fn jit_load_exclusive_rejects_device_reads() {
    let mut machine = Machine::new(1);

    let err = jit_load_exclusive_from_machine(&mut machine, 0, UART_BASE, 4, &[])
        .expect_err("exclusive load helper must reject MMIO");

    assert!(err.contains("device PA"), "{err}");
}
