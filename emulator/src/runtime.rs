//! Multi-core VM runtime orchestrator.
//!
//! Each core runs one instruction at a time in round-robin order, with one
//! decode cache per core and a shared system bus.

use crate::arch::arm64::gic_sysregs::handle_gic_sysreg_access;
use crate::arch::arm64::{
    Armv8Cpu, CpuLifecycle, DecodeCache, Instr, Opcode, decode, execute, translate,
    try_execute_local,
};
use crate::constants::*;
use crate::observability::*;
use crate::platform::virt::SystemBus;

mod boot_context;
pub(crate) mod exceptions;
mod external_io;
#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
mod parallel_native;
#[cfg(any(test, all(feature = "wasm", target_arch = "wasm64")))]
#[cfg_attr(test, allow(dead_code))]
mod parallel_wasm;
mod psci;
mod run;
mod run_faults;
mod run_flow;
mod run_instruction_trace;
mod run_trace_fetch;
mod run_traps;
mod scheduling;
mod simd_opcode_fp;
mod simd_opcode_int;
mod simd_opcode_sve_mem;
mod simd_traps;
#[cfg(any(test, feature = "wasm"))]
#[path = "runtime/parallel_wasm/access.rs"]
mod wasm_access;

use exceptions::*;
use simd_opcode_fp::*;
use simd_opcode_int::*;
use simd_opcode_sve_mem::*;
use simd_traps::*;

pub use boot_context::BootContext;
#[cfg(any(test, feature = "wasm"))]
pub(crate) use wasm_access::WasmAccessControl;
#[cfg(feature = "wasm")]
pub(crate) use wasm_access::WasmIdleAccess;
#[cfg(any(test, target_arch = "wasm64"))]
pub(crate) use wasm_access::{WasmDropAccess, WasmParallelStart};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunBackend {
    Serial,
    NativeThreads,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ParallelRunStats {
    pub worker_threads: usize,
    pub max_local_in_flight: usize,
}

/// Multi-core virtual machine with shared memory bus.
pub struct Machine {
    pub cpus: Vec<Armv8Cpu>,
    pub bus: SystemBus,
    caches: Vec<DecodeCache>,
    trace: TraceState,
    pub active_core: usize,
    pub total_steps: u64,
    pub virtual_time: u64,
    pub fetch_faults: u64,
    pub exec_faults: u64,
    reset_entry: u64,
    reset_arg0: u64,
    reset_memory: Vec<(u64, Vec<u8>)>,
    run_backend: RunBackend,
    memory_epoch: u64,
    parallel_stats: ParallelRunStats,
    #[cfg(any(test, all(feature = "wasm", target_arch = "wasm64")))]
    wasm_parallel: parallel_wasm::WasmParallelControl,
}

impl Machine {
    /// Create a machine with `num_cores` CPUs sharing a single system bus.
    pub fn new(num_cores: usize) -> Self {
        Self::with_trace_options(num_cores, TraceOptions::from_env())
    }

    pub(crate) fn with_trace_options(num_cores: usize, trace_options: TraceOptions) -> Self {
        assert!(num_cores > 0, "Machine requires at least one CPU");
        assert!(
            num_cores <= GICR_MAX_CPUS,
            "Machine CPU count exceeds the redistributor MMIO aperture"
        );
        let mut cpus: Vec<_> = (0..num_cores)
            .map(|i| Armv8Cpu::with_core(i as u32))
            .collect();
        for cpu in cpus.iter_mut().skip(1) {
            cpu.lifecycle = CpuLifecycle::PoweredOff;
        }
        let caches = (0..num_cores).map(|_| DecodeCache::new()).collect();
        Self {
            cpus,
            bus: SystemBus::with_cpu_count(num_cores),
            caches,
            trace: TraceState::new(num_cores, trace_options),
            active_core: 0,
            total_steps: 0,
            virtual_time: 0,
            fetch_faults: 0,
            exec_faults: 0,
            reset_entry: KERNEL_LOAD_ADDR,
            reset_arg0: DTB_BASE,
            reset_memory: Vec::new(),
            run_backend: RunBackend::Serial,
            memory_epoch: 0,
            parallel_stats: ParallelRunStats::default(),
            #[cfg(any(test, all(feature = "wasm", target_arch = "wasm64")))]
            wasm_parallel: parallel_wasm::WasmParallelControl::new(num_cores),
        }
    }

    pub fn set_run_backend(&mut self, backend: RunBackend) {
        self.run_backend = backend;
    }

    pub const fn run_backend(&self) -> RunBackend {
        self.run_backend
    }

    pub const fn parallel_run_stats(&self) -> ParallelRunStats {
        self.parallel_stats
    }

    pub(crate) fn configure_system_reset(
        &mut self,
        entry: u64,
        arg0: u64,
        memory: Vec<(u64, Vec<u8>)>,
    ) {
        self.reset_entry = entry;
        self.reset_arg0 = arg0;
        self.reset_memory = memory;
    }

    pub fn core(&self, n: usize) -> &Armv8Cpu {
        &self.cpus[n]
    }

    pub fn core_mut(&mut self, n: usize) -> &mut Armv8Cpu {
        &mut self.cpus[n]
    }
}

#[cfg(test)]
mod tests;
