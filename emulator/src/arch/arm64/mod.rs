//! ARM64 (AArch64) CPU core.

use crate::constants::{EXCLUSIVE_RESERVATION_GRANULE_BYTES, PSTATE_DAIF_MASK, mpidr_for_core};

mod bitmask_imm;
mod decode;
mod decode_cache;
mod execute;
pub(crate) mod gic_sysregs;
mod helpers;
mod interpreter;
pub mod jit;
mod lifecycle;
pub mod machine;
mod mmu;
mod opcodes;
mod pstate;
mod registers;
mod system_regs;

pub use crate::runtime::Machine;
pub use decode::decode;
pub use decode_cache::DecodeCache;
pub use execute::{execute, try_execute_local};
pub use helpers::{cond_taken, read_base, read_reg, write_reg, write_reg_sp};
pub use interpreter::{RunError, run};
pub use lifecycle::CpuLifecycle;
pub(crate) use mmu::translate_read_only;
#[cfg(feature = "wasm")]
pub(crate) use mmu::translate_write;
pub use mmu::{Tlb, translate};
pub use opcodes::{Instr, Opcode};
pub use pstate::ProcessorState;
pub use registers::RegisterFile;
pub use system_regs::SystemRegisters;

/// ARM64 CPU: combines register file, processor state, system registers, and TLB.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Armv8Cpu {
    pub core_id: u32,
    pub lifecycle: CpuLifecycle,
    pub regs: RegisterFile,
    pub pstate: ProcessorState,
    pub sys: SystemRegisters,
    pub tlb: Tlb,
    pub simd: [u128; 32],
    pub sve_z: [[u8; 256]; 32],
    pub sve_pred: [[u64; 4]; 16],
    pub sve_vl_bytes: u16,
    pub sme_svl_bytes: u16,
    pub exclusive: Option<ExclusiveReservation>,
    pub(crate) exclusive_epoch: u64,
    pub trace_syscall_stack_top: u64,
    pub trace_syscall_access_budget: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExclusiveReservation {
    pub addr: u64,
    pub size: u8,
}

impl Armv8Cpu {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_core(core_id: u32) -> Self {
        let mut cpu = Self {
            core_id,
            ..Self::default()
        };
        cpu.sys.mpidr_el1 = mpidr_for_core(core_id);
        cpu
    }

    /// Reset architectural state without changing CPU identity or lifecycle.
    pub fn reset(&mut self) {
        let core_id = self.core_id;
        let lifecycle = self.lifecycle;
        *self = Self::with_core(core_id);
        self.lifecycle = lifecycle;
    }

    pub fn reserve_exclusive(&mut self, addr: u64, size: u8) {
        self.exclusive = Some(ExclusiveReservation { addr, size });
    }

    pub fn clear_exclusive(&mut self) {
        self.exclusive = None;
        self.exclusive_epoch = 0;
    }

    pub fn exclusive_matches(&self, addr: u64, size: u8) -> bool {
        self.exclusive
            .is_some_and(|reservation| reservation.addr == addr && reservation.size == size)
    }

    pub fn clear_exclusive_if_overlaps(&mut self, addr: u64, size: u8) {
        self.clear_exclusive_range_if_overlaps(addr, size as u64);
    }

    pub fn clear_exclusive_range_if_overlaps(&mut self, addr: u64, len: u64) {
        if self.exclusive.is_some_and(|reservation| {
            let granule_mask = EXCLUSIVE_RESERVATION_GRANULE_BYTES - 1;
            let monitor_start = reservation.addr & !granule_mask;
            let reservation_last = reservation
                .addr
                .saturating_add(reservation.size.saturating_sub(1) as u64);
            let monitor_last = reservation_last & !granule_mask;
            let monitor_len = monitor_last
                .saturating_sub(monitor_start)
                .saturating_add(EXCLUSIVE_RESERVATION_GRANULE_BYTES);
            ranges_overlap(monitor_start, monitor_len, addr, len)
        }) {
            self.clear_exclusive();
        }
    }

    pub fn enter_el1_exception(&mut self, from_lower_el: bool) {
        if from_lower_el {
            self.sys.sp_el0 = self.regs.sp;
            self.regs.sp = self.sys.sp_el1;
        }
        let target = self.pstate.with_el(1).with_daif(PSTATE_DAIF_MASK);
        self.pstate = if from_lower_el {
            target.with_sp_select(true)
        } else {
            target
        };
    }

    pub fn eret_to(&mut self, target: ProcessorState) {
        self.clear_exclusive();
        if target.el() == 0 {
            self.sys.sp_el1 = self.regs.sp;
            self.regs.sp = self.sys.sp_el0;
        }
        self.pstate = target;
    }
}

impl Default for Armv8Cpu {
    fn default() -> Self {
        Self {
            core_id: 0,
            lifecycle: CpuLifecycle::default(),
            regs: RegisterFile::default(),
            pstate: ProcessorState::new(),
            sys: SystemRegisters::default(),
            tlb: Tlb::default(),
            simd: [0; 32],
            sve_z: [[0; 256]; 32],
            sve_pred: [[0; 4]; 16],
            sve_vl_bytes: 16,
            sme_svl_bytes: 16,
            exclusive: None,
            exclusive_epoch: 0,
            trace_syscall_stack_top: 0,
            trace_syscall_access_budget: 0,
        }
    }
}

fn ranges_overlap(a: u64, a_size: u64, b: u64, b_size: u64) -> bool {
    let a_end = a.saturating_add(a_size);
    let b_end = b.saturating_add(b_size);
    a < b_end && b < a_end
}

#[cfg(test)]
mod tests;
