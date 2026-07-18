use crate::arch::arm64::jit::WasmJitCpuState;
use crate::runtime::Machine;

use super::commit::commit_jit_state;
use super::timer::deliver_jit_timer_boundary;

pub(super) fn commit_finished_jit_state(
    state: &WasmJitCpuState,
    machine: &mut Machine,
    core_id: usize,
    steps: usize,
    expected_exit_pc: u64,
    prepared: bool,
) -> Result<(), String> {
    if !prepared {
        return commit_jit_state(state, machine, core_id, steps, expected_exit_pc);
    }
    commit_prepared_jit_state(state, machine, core_id, steps, expected_exit_pc)
}

fn commit_prepared_jit_state(
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
    if machine.cpus.len() != 1 {
        return Err("JIT commit is currently restricted to single-core VMs".to_string());
    }
    if machine.active_core != core_id {
        return Err(format!(
            "JIT core mismatch: active core is {}, requested {core_id}",
            machine.active_core
        ));
    }

    let cpu = &mut machine.cpus[core_id];
    let cycle_count = cpu.sys.cycle_count;
    state.copy_to_cpu(cpu);
    cpu.sys.cycle_count = cycle_count.wrapping_add(steps as u64);
    machine.virtual_time = machine.virtual_time.max(cpu.sys.cycle_count);
    deliver_jit_timer_boundary(cpu);
    machine.total_steps = machine.total_steps.wrapping_add(steps as u64);
    machine.active_core = 0;
    Ok(())
}
