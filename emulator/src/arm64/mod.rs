//! ARM64 (AArch64) CPU core.

mod bitmask_imm;
mod decode;
mod decode_cache;
mod execute;
mod helpers;
mod jit;
mod mmu;
mod opcodes;
mod interpreter;
mod pstate;
mod registers;
mod system_regs;

pub use decode::decode;
pub use decode_cache::DecodeCache;
pub use execute::execute;
pub use helpers::{cond_taken, read_reg, read_base, write_reg};
pub use mmu::{Tlb, translate};
pub use opcodes::{Instr, Opcode};
pub use interpreter::{run, RunError};
pub use pstate::ProcessorState;
pub use registers::RegisterFile;
pub use system_regs::SystemRegisters;

/// ARM64 CPU: combines register file, processor state, system registers, and TLB.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Armv8Cpu {
    pub regs: RegisterFile,
    pub pstate: ProcessorState,
    pub sys: SystemRegisters,
    pub tlb: Tlb,
}

impl Armv8Cpu {
    pub fn new() -> Self { Self::default() }
    pub fn reset(&mut self) { *self = Self::default(); }
}

impl Default for Armv8Cpu {
    fn default() -> Self {
        Self {
            regs: RegisterFile::default(),
            pstate: ProcessorState::new(),
            sys: SystemRegisters::default(),
            tlb: Tlb::default(),
        }
    }
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
