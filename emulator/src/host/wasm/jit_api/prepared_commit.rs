use crate::arch::arm64::jit::WasmJitCpuState;
use crate::runtime::Machine;

use super::commit::{
    apply_committed_jit_state, finish_committed_jit_state, validate_jit_state_commit,
};
use super::commit_boundary::validate_jit_commit_target;

pub(super) fn commit_finished_jit_state(
    state: &WasmJitCpuState,
    machine: &mut Machine,
    core_id: usize,
    steps: usize,
    expected_exit_pc: u64,
    prepared: bool,
) -> Result<(), String> {
    apply_finished_jit_state(state, machine, core_id, steps, expected_exit_pc, prepared)?;
    finish_committed_jit_state(machine, core_id, steps);
    Ok(())
}

pub(super) fn apply_finished_jit_state(
    state: &WasmJitCpuState,
    machine: &mut Machine,
    core_id: usize,
    steps: usize,
    expected_exit_pc: u64,
    prepared: bool,
) -> Result<(), String> {
    if !prepared {
        validate_jit_state_commit(state, machine, core_id, steps, expected_exit_pc)?;
        apply_committed_jit_state(state, machine, core_id, steps);
        return Ok(());
    }
    if steps == 0 {
        return Err("cannot commit an empty JIT block".to_string());
    }
    if state.pc != expected_exit_pc {
        return Err(format!(
            "JIT block exit mismatch: expected=0x{expected_exit_pc:016x} actual=0x{:016x}",
            state.pc
        ));
    }
    validate_jit_commit_target(machine, core_id, steps)?;
    apply_committed_jit_state(state, machine, core_id, steps);
    Ok(())
}
