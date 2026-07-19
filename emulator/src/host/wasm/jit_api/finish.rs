use crate::arch::arm64::jit::WasmJitCpuState;
use crate::host::wasm::{Emulator, JitPendingExclusiveReservation, JitPendingStore};
use crate::runtime::Machine;
use wasm_bindgen::prelude::*;

use super::commit::finish_committed_jit_state;
use super::exclusive::apply_jit_pending_exclusive_clear;
use super::exclusive_load::apply_jit_pending_exclusive_reservation;
use super::prepared_commit::{apply_finished_jit_state, commit_finished_jit_state};
use super::store::apply_jit_pending_stores;

pub(super) const JIT_FINISH_COMMITTED: u8 = 0;
pub(super) const JIT_FINISH_HELPER_REJECTED: u8 = 1;
pub(super) const JIT_FINISH_COMMIT_REJECTED: u8 = 2;
pub(super) const JIT_FINISH_EXIT_REJECTED: u8 = 3;
const ANY_DYNAMIC_EXIT_PC: u64 = u64::MAX;

#[wasm_bindgen]
impl Emulator {
    /// Finish a generated JIT block run and commit when no helper rejected.
    pub fn jit_finish_cached_block(
        &mut self,
        core_id: Option<usize>,
        steps: usize,
        exit_pc: u64,
        expected_exit_pc: u64,
        alternate_exit_pc: u64,
        dynamic_exit: bool,
    ) -> u8 {
        let _access = self.require_parallel_idle();
        if self.jit_helper_failed {
            self.clear_jit_side_effects();
            return JIT_FINISH_HELPER_REJECTED;
        }
        if let Err(err) =
            validate_jit_exit(exit_pc, expected_exit_pc, alternate_exit_pc, dynamic_exit)
        {
            self.jit_last_error = err;
            self.clear_jit_side_effects();
            return JIT_FINISH_EXIT_REJECTED;
        }

        let core_id = core_id.unwrap_or(0);
        let prepared = std::mem::take(&mut self.jit_prepared_block);
        let no_side_effects = self.jit_pending_stores.is_empty()
            && self.jit_pending_exclusive_clear.is_none()
            && self.jit_pending_exclusive_reservation.is_none();
        if no_side_effects {
            let result = if let Some(ref mut boot) = self.boot {
                commit_finished_jit_state(
                    &self.jit_state,
                    &mut boot.machine,
                    core_id,
                    steps,
                    exit_pc,
                    prepared,
                )
            } else {
                commit_finished_jit_state(
                    &self.jit_state,
                    &mut self.machine,
                    core_id,
                    steps,
                    exit_pc,
                    prepared,
                )
            };
            return match result {
                Ok(()) => {
                    self.jit_last_error.clear();
                    JIT_FINISH_COMMITTED
                }
                Err(err) => {
                    self.jit_last_error = err;
                    JIT_FINISH_COMMIT_REJECTED
                }
            };
        }

        let pending_stores = std::mem::take(&mut self.jit_pending_stores);
        let pending_exclusive_clear = self.jit_pending_exclusive_clear.take();
        let pending_exclusive_reservation = self.jit_pending_exclusive_reservation.take();
        let result = if let Some(ref mut boot) = self.boot {
            finish_jit_block(
                &self.jit_state,
                &mut boot.machine,
                core_id,
                steps,
                exit_pc,
                &pending_stores,
                pending_exclusive_clear,
                pending_exclusive_reservation,
                prepared,
            )
        } else {
            finish_jit_block(
                &self.jit_state,
                &mut self.machine,
                core_id,
                steps,
                exit_pc,
                &pending_stores,
                pending_exclusive_clear,
                pending_exclusive_reservation,
                prepared,
            )
        };

        match result {
            Ok(()) => {
                self.jit_last_error.clear();
                JIT_FINISH_COMMITTED
            }
            Err(err) => {
                self.jit_last_error = err;
                JIT_FINISH_COMMIT_REJECTED
            }
        }
    }
}

pub(super) fn finish_jit_block(
    state: &WasmJitCpuState,
    machine: &mut Machine,
    core_id: usize,
    steps: usize,
    expected_exit_pc: u64,
    pending_stores: &[JitPendingStore],
    pending_exclusive_clear: Option<usize>,
    pending_exclusive_reservation: Option<JitPendingExclusiveReservation>,
    prepared: bool,
) -> Result<(), String> {
    apply_finished_jit_state(state, machine, core_id, steps, expected_exit_pc, prepared)?;
    apply_jit_pending_stores(machine, pending_stores)?;
    apply_jit_pending_exclusive_clear(machine, pending_exclusive_clear);
    apply_jit_pending_exclusive_reservation(machine, pending_exclusive_reservation);
    finish_committed_jit_state(machine, core_id, steps);
    Ok(())
}

fn validate_jit_exit(
    exit_pc: u64,
    expected_exit_pc: u64,
    alternate_exit_pc: u64,
    dynamic_exit: bool,
) -> Result<(), String> {
    if exit_pc == expected_exit_pc
        || (dynamic_exit && alternate_exit_pc == ANY_DYNAMIC_EXIT_PC)
        || (dynamic_exit && exit_pc == alternate_exit_pc)
    {
        return Ok(());
    }
    let actual = format!("0x{exit_pc:x}");
    let expected = format!("0x{expected_exit_pc:x}");
    if !dynamic_exit {
        return Err(format!("JIT block returned {actual} instead of {expected}"));
    }
    if alternate_exit_pc == ANY_DYNAMIC_EXIT_PC {
        return Err(format!(
            "JIT block returned {actual} outside arbitrary dynamic exit"
        ));
    }
    Err(format!(
        "JIT block returned {actual} outside {expected}/0x{alternate_exit_pc:x}"
    ))
}
