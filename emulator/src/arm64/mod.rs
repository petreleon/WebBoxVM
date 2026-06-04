//! ARM64 (AArch64) CPU core.

use crate::constants::PSTATE_DAIF_MASK;

mod bitmask_imm;
mod decode;
mod decode_cache;
mod execute;
mod gic_sysregs;
mod helpers;
mod interpreter;
pub mod jit;
pub mod machine;
mod machine_trace;
mod mmu;
mod opcodes;
mod pstate;
mod registers;
mod system_regs;

pub use decode::decode;
pub use decode_cache::DecodeCache;
pub use execute::execute;
pub use helpers::{cond_taken, read_base, read_reg, write_reg, write_reg_sp};
pub use interpreter::{RunError, run};
pub use machine::Machine;
pub use mmu::{Tlb, translate};
pub use opcodes::{Instr, Opcode};
pub use pstate::ProcessorState;
pub use registers::RegisterFile;
pub use system_regs::SystemRegisters;

/// ARM64 CPU: combines register file, processor state, system registers, and TLB.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Armv8Cpu {
    pub core_id: u32,
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
        Self {
            core_id,
            ..Self::default()
        }
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn reserve_exclusive(&mut self, addr: u64, size: u8) {
        self.exclusive = Some(ExclusiveReservation { addr, size });
    }

    pub fn clear_exclusive(&mut self) {
        self.exclusive = None;
    }

    pub fn exclusive_matches(&self, addr: u64, size: u8) -> bool {
        self.exclusive
            .is_some_and(|reservation| reservation.addr == addr && reservation.size == size)
    }

    pub fn clear_exclusive_if_overlaps(&mut self, addr: u64, size: u8) {
        if self.exclusive.is_some_and(|reservation| {
            ranges_overlap(reservation.addr, reservation.size, addr, size)
        }) {
            self.clear_exclusive();
        }
    }

    pub fn enter_el1_exception(&mut self, from_lower_el: bool) {
        if from_lower_el {
            self.sys.sp_el0 = self.regs.sp;
            self.regs.sp = self.sys.sp_el1;
        }
        self.pstate = self.pstate.with_el(1).with_daif(PSTATE_DAIF_MASK);
    }

    pub fn eret_to(&mut self, target: ProcessorState) {
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
            trace_syscall_stack_top: 0,
            trace_syscall_access_budget: 0,
        }
    }
}

fn ranges_overlap(a: u64, a_size: u8, b: u64, b_size: u8) -> bool {
    let a_end = a.saturating_add(a_size as u64);
    let b_end = b.saturating_add(b_size as u64);
    a < b_end && b < a_end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_state() {
        let cpu = Armv8Cpu::new();
        assert_eq!(cpu.pstate.el(), 3);
        assert_eq!(cpu.regs.x(0), 0);
        assert_eq!(cpu.sys.sctlr_el1, 0);
    }

    #[test]
    fn reset_clears_all() {
        let mut cpu = Armv8Cpu::new();
        cpu.regs.set_x(0, 42);
        cpu.reset();
        assert_eq!(cpu.regs.x(0), 0);
    }
}
