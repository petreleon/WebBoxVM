use crate::arch::arm64::jit::WasmJitCpuState;
use crate::constants::GIC_SPURIOUS_INTERRUPT;
use crate::host::wasm::Emulator;
use crate::runtime::Machine;
use wasm_bindgen::prelude::*;

use super::store::apply_jit_pending_stores;

#[wasm_bindgen]
impl Emulator {
    /// Commit the JIT state buffer back to a core after a generated block runs.
    ///
    /// This is deliberately conservative. It only commits for single-core VMs
    /// and rejects blocks that could cross a timer deadline or pending unmasked
    /// IRQ boundary.
    pub fn jit_commit_state_to_core(
        &mut self,
        core_id: Option<usize>,
        steps: usize,
        expected_exit_pc: u64,
    ) -> bool {
        let core_id = core_id.unwrap_or(0);
        let pending_stores = std::mem::take(&mut self.jit_pending_stores);
        let result = if let Some(ref mut boot) = self.boot {
            commit_jit_state(
                &self.jit_state,
                &mut boot.machine,
                core_id,
                steps,
                expected_exit_pc,
            )
            .and_then(|()| apply_jit_pending_stores(&mut boot.machine, &pending_stores))
        } else {
            commit_jit_state(
                &self.jit_state,
                &mut self.machine,
                core_id,
                steps,
                expected_exit_pc,
            )
            .and_then(|()| apply_jit_pending_stores(&mut self.machine, &pending_stores))
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

pub(super) fn commit_jit_state(
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

    let steps = steps as u64;
    {
        let Some(cpu) = machine.cpus.get(core_id) else {
            return Err(format!("core {core_id} does not exist"));
        };

        if let Some(deadline) = cpu.sys.next_timer_deadline() {
            let end_cycle = cpu.sys.cycle_count.saturating_add(steps);
            if deadline <= end_cycle {
                return Err(format!(
                    "JIT block crosses timer deadline at cycle {deadline}"
                ));
            }
        }

        machine.bus.refresh_interrupts();
        let external_irq = machine.bus.gic.next_pending_enabled();
        let cpu_irq = cpu.sys.irq_pending && cpu.sys.last_irq_id != GIC_SPURIOUS_INTERRUPT as u32;
        if !cpu.pstate.irq_masked() && (cpu_irq || external_irq.is_some()) {
            return Err("JIT block crosses an unmasked pending IRQ boundary".to_string());
        }
    }

    let cpu = &mut machine.cpus[core_id];
    let cycle_count = cpu.sys.cycle_count;
    state.copy_to_cpu(cpu);
    cpu.sys.cycle_count = cycle_count.wrapping_add(steps);
    machine.total_steps = machine.total_steps.wrapping_add(steps);
    machine.active_core = (core_id + 1) % machine.cpus.len();
    Ok(())
}
