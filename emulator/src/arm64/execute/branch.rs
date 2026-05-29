//! Branch instruction helpers.

use crate::arm64::helpers::{read_reg, write_reg};
use crate::arm64::Armv8Cpu;
use crate::constants::LINK_REGISTER_INDEX;

pub(super) fn branch_target(pc: u64, offset: u64) -> u64 {
    (pc as i64 + offset as i64) as u64
}

/// Simple relative branch (B, B.cond, CBZ, CBNZ, TBZ, TBNZ).
pub(super) fn branch(cpu: &mut Armv8Cpu, offset: u64) -> Result<(), &'static str> {
    cpu.regs.pc = branch_target(cpu.regs.pc, offset);
    Ok(())
}

/// Branch with Link (BL) — sets LR=X30 before jumping.
pub(super) fn branch_link(cpu: &mut Armv8Cpu, offset: u64) -> Result<(), &'static str> {
    write_reg(cpu, LINK_REGISTER_INDEX, cpu.regs.pc + 4, true);
    cpu.regs.pc = branch_target(cpu.regs.pc, offset);
    Ok(())
}

/// Register branch (BR, RET).
pub(super) fn branch_reg(cpu: &mut Armv8Cpu, rn: u8) -> Result<(), &'static str> {
    cpu.regs.pc = read_reg(cpu, rn, true);
    Ok(())
}

/// Register branch with link (BLR).
pub(super) fn branch_link_reg(cpu: &mut Armv8Cpu, rn: u8) -> Result<(), &'static str> {
    write_reg(cpu, LINK_REGISTER_INDEX, cpu.regs.pc + 4, true);
    cpu.regs.pc = read_reg(cpu, rn, true);
    Ok(())
}
