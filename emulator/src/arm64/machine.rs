//! Multi-core ARM64 machine orchestrator.
//!
//! Each core runs one instruction at a time in round-robin order, with one
//! decode cache per core and a shared system bus.

use crate::arm64::gic_sysregs::handle_gic_sysreg_access;
use crate::arm64::machine_trace::{TraceOptions, TraceState, TraceSyscall};
use crate::arm64::{Armv8Cpu, DecodeCache, Instr, Opcode, cond_taken, decode, execute, translate};
use crate::bus::SystemBus;
use crate::constants::*;

pub(crate) mod exceptions;
mod run;
mod run_faults;
mod run_flow;
mod run_instruction_trace;
mod run_trace_fetch;
mod run_traps;
mod simd_opcode_fp;
mod simd_opcode_int;
mod simd_opcode_sve_mem;
mod simd_traps;
mod trace_hotspots;
mod trace_memory;
mod trace_paths;
mod trace_stack;
mod trace_syscalls;
mod trace_syscalls_exec;

use exceptions::*;
use simd_opcode_fp::*;
use simd_opcode_int::*;
use simd_opcode_sve_mem::*;
use simd_traps::*;
use trace_hotspots::*;
use trace_memory::*;
use trace_paths::*;
use trace_stack::*;
use trace_syscalls::*;
use trace_syscalls_exec::*;

/// Multi-core virtual machine with shared memory bus.
pub struct Machine {
    pub cpus: Vec<Armv8Cpu>,
    pub bus: SystemBus,
    caches: Vec<DecodeCache>,
    trace: TraceState,
    pub active_core: usize,
    pub total_steps: u64,
    pub fetch_faults: u64,
    pub exec_faults: u64,
}

impl Machine {
    /// Create a machine with `num_cores` CPUs sharing a single system bus.
    pub fn new(num_cores: usize) -> Self {
        Self::with_trace_options(num_cores, TraceOptions::from_env())
    }

    pub(crate) fn with_trace_options(num_cores: usize, trace_options: TraceOptions) -> Self {
        let cpus: Vec<_> = (0..num_cores)
            .map(|i| Armv8Cpu::with_core(i as u32))
            .collect();
        let caches = (0..num_cores).map(|_| DecodeCache::new()).collect();
        Self {
            cpus,
            bus: SystemBus::new(),
            caches,
            trace: TraceState::new(num_cores, trace_options),
            active_core: 0,
            total_steps: 0,
            fetch_faults: 0,
            exec_faults: 0,
        }
    }

    pub fn core(&self, n: usize) -> &Armv8Cpu {
        &self.cpus[n]
    }

    pub fn core_mut(&mut self, n: usize) -> &mut Armv8Cpu {
        &mut self.cpus[n]
    }

    pub fn inject_irq(&mut self, int_id: u32) {
        self.bus.gic.set_pending(int_id);
    }
}

#[cfg(test)]
mod tests;
