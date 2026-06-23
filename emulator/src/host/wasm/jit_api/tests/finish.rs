use super::*;
use crate::arch::arm64::jit::WasmJitCpuState;
use crate::host::wasm::jit_api::finish::{
    JIT_FINISH_COMMIT_REJECTED, JIT_FINISH_COMMITTED, JIT_FINISH_EXIT_REJECTED,
    JIT_FINISH_HELPER_REJECTED, finish_jit_block,
};
use crate::host::wasm::{Emulator, JitPendingStore};

#[test]
fn finish_jit_block_applies_pending_store_on_commit() {
    let mut machine = Machine::new(1);
    let store_pa = RAM_BASE + 0x80;
    machine.bus.mem.write(store_pa, 4, 0);
    let mut state = WasmJitCpuState::from_cpu(&machine.cpus[0]);
    state.pc = RAM_BASE + 4;
    let stores = [JitPendingStore::new(store_pa, &[0x11, 0x22, 0x33, 0x44])];

    finish_jit_block(
        &state,
        &mut machine,
        0,
        1,
        RAM_BASE + 4,
        &stores,
        None,
        None,
    )
    .expect("finish should commit valid JIT state and stores");

    assert_eq!(machine.cpus[0].regs.pc, RAM_BASE + 4);
    assert_eq!(machine.bus.mem.read(store_pa, 4), Some(0x4433_2211));
}

#[test]
fn finish_cached_block_commits_without_pending_side_effects() {
    let mut emulator = Emulator::new(Some(1));
    emulator.jit_state.pc = RAM_BASE + 4;

    let result = emulator.jit_finish_cached_block(Some(0), 1, RAM_BASE + 4, RAM_BASE + 4, 0, false);

    assert_eq!(result, JIT_FINISH_COMMITTED);
    assert_eq!(emulator.machine.cpus[0].regs.pc, RAM_BASE + 4);
    assert_eq!(emulator.machine.active_core, 0);
    assert!(emulator.jit_pending_stores.is_empty());
    assert!(emulator.jit_pending_exclusive_clear.is_none());
    assert!(emulator.jit_pending_exclusive_reservation.is_none());
}

#[test]
fn finish_cached_block_reports_helper_rejection_without_commit() {
    let mut emulator = Emulator::new(Some(1));
    let store_pa = RAM_BASE + 0x90;
    emulator.machine.bus.mem.write(store_pa, 4, 0);
    emulator.jit_state.pc = RAM_BASE + 4;
    emulator.jit_helper_failed = true;
    emulator.jit_last_error = "JIT helper failed".into();
    emulator
        .jit_pending_stores
        .push(JitPendingStore::new(store_pa, &[1, 2, 3, 4]));

    let result = emulator.jit_finish_cached_block(Some(0), 1, RAM_BASE + 4, RAM_BASE + 4, 0, false);

    assert_eq!(result, JIT_FINISH_HELPER_REJECTED);
    assert_eq!(emulator.machine.cpus[0].regs.pc, 0);
    assert_eq!(emulator.machine.bus.mem.read(store_pa, 4), Some(0));
    assert!(emulator.jit_pending_stores.is_empty());
}

#[test]
fn finish_cached_block_rejects_bad_exit_before_commit() {
    let mut emulator = Emulator::new(Some(1));
    emulator.jit_state.pc = RAM_BASE + 0x20;

    let result =
        emulator.jit_finish_cached_block(Some(0), 1, RAM_BASE + 0x20, RAM_BASE + 0x10, 0, false);

    assert_eq!(result, JIT_FINISH_EXIT_REJECTED);
    assert_eq!(emulator.machine.cpus[0].regs.pc, 0);
    assert!(
        emulator
            .jit_last_error
            .contains("returned 0x40000020 instead of 0x40000010")
    );
}

#[test]
fn finish_cached_block_reports_commit_rejection() {
    let mut emulator = Emulator::new(Some(1));
    emulator.jit_state.pc = RAM_BASE + 4;

    let result = emulator.jit_finish_cached_block(Some(0), 0, RAM_BASE + 4, RAM_BASE + 4, 0, false);

    assert_eq!(result, JIT_FINISH_COMMIT_REJECTED);
    assert_ne!(result, JIT_FINISH_COMMITTED);
    assert!(emulator.jit_last_error.contains("empty JIT block"));
}
