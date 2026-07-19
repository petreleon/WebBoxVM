use super::commit_boundary::can_commit_jit_block_now;
use super::exclusive::apply_jit_pending_exclusive_clear;
use super::exclusive_load::apply_jit_pending_exclusive_reservation;
use super::store::apply_jit_pending_stores;
use super::timer::deliver_jit_timer_boundary;
use crate::arch::arm64::jit::WasmJitCpuState;
use crate::host::wasm::{Emulator, JitPendingExclusiveReservation, JitPendingStore};
use crate::runtime::Machine;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Emulator {
    /// Check whether a generated JIT block can commit at the current boundary.
    pub fn jit_can_commit_block_now(&mut self, core_id: Option<usize>, steps: usize) -> bool {
        let _access = self.require_parallel_idle();
        let core_id = core_id.unwrap_or(0);
        let result = if let Some(ref mut boot) = self.boot {
            can_commit_jit_block_now(&mut boot.machine, core_id, steps)
        } else {
            can_commit_jit_block_now(&mut self.machine, core_id, steps)
        };

        match result {
            Ok(()) => {
                self.jit_last_error.clear();
                true
            }
            Err(err) => {
                self.jit_last_error = err;
                false
            }
        }
    }

    /// Commit the JIT state buffer back to a core after a generated block runs.
    ///
    /// This is deliberately conservative. It rejects blocks that could cross a
    /// timer deadline, scheduler wake deadline, or pending unmasked IRQ boundary.
    pub fn jit_commit_state_to_core(
        &mut self,
        core_id: Option<usize>,
        steps: usize,
        expected_exit_pc: u64,
    ) -> bool {
        let _access = self.require_parallel_idle();
        let core_id = core_id.unwrap_or(0);
        self.jit_prepared_block = false;
        let pending_stores = std::mem::take(&mut self.jit_pending_stores);
        let pending_exclusive_clear = self.jit_pending_exclusive_clear.take();
        let pending_exclusive_reservation = self.jit_pending_exclusive_reservation.take();
        let result = if let Some(ref mut boot) = self.boot {
            commit_jit_state_with_side_effects(
                &self.jit_state,
                &mut boot.machine,
                core_id,
                steps,
                expected_exit_pc,
                &pending_stores,
                pending_exclusive_clear,
                pending_exclusive_reservation,
            )
        } else {
            commit_jit_state_with_side_effects(
                &self.jit_state,
                &mut self.machine,
                core_id,
                steps,
                expected_exit_pc,
                &pending_stores,
                pending_exclusive_clear,
                pending_exclusive_reservation,
            )
        };

        match result {
            Ok(()) => {
                self.jit_last_error.clear();
                true
            }
            Err(err) => {
                self.jit_last_error = err;
                false
            }
        }
    }
}

#[cfg(test)]
pub(super) fn commit_jit_state(
    state: &WasmJitCpuState,
    machine: &mut Machine,
    core_id: usize,
    steps: usize,
    expected_exit_pc: u64,
) -> Result<(), String> {
    validate_jit_state_commit(state, machine, core_id, steps, expected_exit_pc)?;
    apply_committed_jit_state(state, machine, core_id, steps);
    finish_committed_jit_state(machine, core_id, steps);
    Ok(())
}

fn commit_jit_state_with_side_effects(
    state: &WasmJitCpuState,
    machine: &mut Machine,
    core_id: usize,
    steps: usize,
    expected_exit_pc: u64,
    pending_stores: &[JitPendingStore],
    pending_exclusive_clear: Option<usize>,
    pending_exclusive_reservation: Option<JitPendingExclusiveReservation>,
) -> Result<(), String> {
    validate_jit_state_commit(state, machine, core_id, steps, expected_exit_pc)?;
    apply_committed_jit_state(state, machine, core_id, steps);
    apply_jit_pending_stores(machine, pending_stores)?;
    apply_jit_pending_exclusive_clear(machine, pending_exclusive_clear);
    apply_jit_pending_exclusive_reservation(machine, pending_exclusive_reservation);
    finish_committed_jit_state(machine, core_id, steps);
    Ok(())
}

pub(super) fn validate_jit_state_commit(
    state: &WasmJitCpuState,
    machine: &mut Machine,
    core_id: usize,
    steps: usize,
    expected_exit_pc: u64,
) -> Result<(), String> {
    if steps == 0 {
        return Err("cannot commit an empty JIT block".to_string());
    }
    if state.pc != expected_exit_pc {
        return Err(format!(
            "JIT block exit mismatch: expected=0x{expected_exit_pc:016x} actual=0x{:016x}",
            state.pc
        ));
    }
    can_commit_jit_block_now(machine, core_id, steps)?;
    Ok(())
}

pub(super) fn apply_committed_jit_state(
    state: &WasmJitCpuState,
    machine: &mut Machine,
    core_id: usize,
    steps: usize,
) {
    let steps = steps as u64;
    let cpu = &mut machine.cpus[core_id];
    let cycle_count = cpu.sys.cycle_count;
    state.copy_to_cpu(cpu);
    cpu.sys.cycle_count = cycle_count.wrapping_add(steps);
}

pub(super) fn finish_committed_jit_state(machine: &mut Machine, core_id: usize, steps: usize) {
    let steps = steps as u64;
    let cpu = &mut machine.cpus[core_id];
    deliver_jit_timer_boundary(cpu);
    machine.finish_jit_core(core_id, steps);
}
