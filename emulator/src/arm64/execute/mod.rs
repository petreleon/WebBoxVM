//! Instruction execution engine — mutates CPU and bus state for every ARM64 instruction.
//!
//! For each decoded instruction, this module:
//!   1. Reads source registers (handling XZR/WZR semantics)
//!   2. Performs the operation (ALU, load/store, branch, etc.)
//!   3. Writes the result to the destination register
//!   4. Increments PC and the cycle counter
//!   5. Checks for timer interrupt delivery

mod branch;
mod load_store;
mod system;
mod alu;
mod debug;

pub(super) use super::opcodes::{Instr, Opcode};
use branch::{branch, branch_link, branch_reg, branch_link_reg, branch_target};
use load_store::{exec_ldr_str, exec_ldr_lit, exec_ldp_stp, exec_exclusive};
use system::{exec_msr, exec_svc, exec_eret, exec_brk};
use alu::*;

use super::helpers::{cond_taken, read_reg, read_base, write_reg, write_reg_sp};
use super::Armv8Cpu;
use crate::bus::SystemBus;
use crate::arm64::mmu::translate;
use crate::constants::*;

/// Execute one decoded instruction, returning an error string if something goes wrong.
pub fn execute(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    match instr.op {
        Opcode::Add  => write_reg_sp(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf).wrapping_add(shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf)), instr.sf),
        Opcode::Sub  => write_reg_sp(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf).wrapping_sub(shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf)), instr.sf),
        Opcode::Adds => { let lhs = read_reg(cpu, instr.rn, instr.sf); let rhs = shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf); let val = add_flags(cpu, lhs, rhs, instr.sf); if instr.rd != ZERO_REGISTER_INDEX { write_reg_sp(cpu, instr.rd, val, instr.sf); } }
        Opcode::Subs => { let lhs = read_reg(cpu, instr.rn, instr.sf); let rhs = shifted_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf); let val = sub_flags(cpu, lhs, rhs, instr.sf); if instr.rd != ZERO_REGISTER_INDEX { write_reg_sp(cpu, instr.rd, val, instr.sf); } }

        Opcode::Movz  => write_reg(cpu, instr.rd, instr.imm, instr.sf),
        Opcode::Movn  => write_reg(cpu, instr.rd, instr.imm, instr.sf),
        Opcode::MovReg => write_reg(cpu, instr.rd, read_reg(cpu, instr.rm, instr.sf), instr.sf),
        Opcode::Sxtw  => { let val = read_reg(cpu, instr.rn, false); write_reg(cpu, instr.rd, ((val as i32) as i64) as u64, true); }
        Opcode::Movk  => { let hw = instr.cond as u64; let mask = !(0xFFFFu64 << (hw * 16)); let old = read_reg(cpu, instr.rd, instr.sf); write_reg(cpu, instr.rd, (old & mask) | instr.imm, instr.sf); }

        Opcode::AddImm   => write_reg_sp(cpu, instr.rd, read_base(cpu, instr.rn, instr.sf).wrapping_add(instr.imm), instr.sf),
        Opcode::SubImm   => write_reg_sp(cpu, instr.rd, read_base(cpu, instr.rn, instr.sf).wrapping_sub(instr.imm), instr.sf),
        Opcode::AddsImm  => { let lhs = read_base(cpu, instr.rn, instr.sf); let val = add_flags(cpu, lhs, instr.imm, instr.sf); if instr.rd != ZERO_REGISTER_INDEX { write_reg_sp(cpu, instr.rd, val, instr.sf); } }
        Opcode::SubsImm  => { let lhs = read_base(cpu, instr.rn, instr.sf); let val = sub_flags(cpu, lhs, instr.imm, instr.sf); if instr.rd != ZERO_REGISTER_INDEX { write_reg_sp(cpu, instr.rd, val, instr.sf); } }
        Opcode::CmpImm   => { let lhs = read_reg(cpu, instr.rn, instr.sf); let _ = sub_flags(cpu, lhs, instr.imm, instr.sf); }
        Opcode::Cmp      => { let lhs = read_reg(cpu, instr.rn, instr.sf); let rhs = ext_or_shifted_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf); let _ = sub_flags(cpu, lhs, rhs, instr.sf); }

        Opcode::Adr  => write_reg(cpu, instr.rd, branch_target(cpu.regs.pc, instr.imm), true),
        Opcode::Adrp => { let page = cpu.regs.pc & !PAGE_OFFSET_MASK; write_reg(cpu, instr.rd, (page as i64 + instr.imm as i64) as u64, true); }

        // ── Load / Store ──
        Opcode::Ldr | Opcode::Str => exec_ldr_str(cpu, bus, instr)?,
        Opcode::LdrLit            => exec_ldr_lit(cpu, bus, instr)?,
        Opcode::Ldp | Opcode::Stp | Opcode::SimdLdp | Opcode::SimdStp => exec_ldp_stp(cpu, bus, instr)?,
        Opcode::Ldxr | Opcode::Ldar | Opcode::Stxr | Opcode::Stlr | Opcode::Ldxp | Opcode::Stxp => exec_exclusive(cpu, bus, instr)?,

        // ── Branches ──
        Opcode::B   => return branch(cpu, instr.imm),
        Opcode::Bl  => return branch_link(cpu, instr.imm),
        Opcode::Blr => return branch_link_reg(cpu, instr.rn),
        Opcode::Br  => return branch_reg(cpu, instr.rn),
        Opcode::Ret => return branch_reg(cpu, instr.rn),
        Opcode::Cbz  => { if read_reg(cpu, instr.rd, instr.sf) == 0 { return branch(cpu, instr.imm); } }
        Opcode::Cbnz => { if read_reg(cpu, instr.rd, instr.sf) != 0 { return branch(cpu, instr.imm); } }
        Opcode::BCond => { if cond_taken(cpu, instr.cond) { return branch(cpu, instr.imm); } }
        Opcode::Tbz  => { if (read_reg(cpu, instr.rd, instr.sf) >> (instr.cond as u64)) & 1 == 0 { return branch(cpu, instr.imm); } }
        Opcode::Tbnz => { if (read_reg(cpu, instr.rd, instr.sf) >> (instr.cond as u64)) & 1 != 0 { return branch(cpu, instr.imm); } }

        // ── Conditional select / compare ──
        Opcode::Csel  => write_reg(cpu, instr.rd, if cond_taken(cpu, instr.cond) { read_reg(cpu, instr.rn, instr.sf) } else { read_reg(cpu, instr.rm, instr.sf) }, instr.sf),
        Opcode::Csinc => write_reg(cpu, instr.rd, if cond_taken(cpu, instr.cond) { read_reg(cpu, instr.rn, instr.sf) } else { read_reg(cpu, instr.rm, instr.sf).wrapping_add(1) }, instr.sf),
        Opcode::Csinv => write_reg(cpu, instr.rd, if cond_taken(cpu, instr.cond) { read_reg(cpu, instr.rn, instr.sf) } else { !read_reg(cpu, instr.rm, instr.sf) }, instr.sf),
        Opcode::Csneg => write_reg(cpu, instr.rd, if cond_taken(cpu, instr.cond) { read_reg(cpu, instr.rn, instr.sf) } else { 0u64.wrapping_sub(read_reg(cpu, instr.rm, instr.sf)) }, instr.sf),
        Opcode::Ccmp  => exec_ccmp(cpu, instr),

        // ── Logical (immediate) ──
        Opcode::AndImm  => write_reg(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf) & instr.imm, instr.sf),
        Opcode::OrrImm  => write_reg(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf) | instr.imm, instr.sf),
        Opcode::EorImm  => write_reg(cpu, instr.rd, read_reg(cpu, instr.rn, instr.sf) ^ instr.imm, instr.sf),
        Opcode::AndsImm => { let val = read_reg(cpu, instr.rn, instr.sf) & instr.imm; set_nz_flags(cpu, val, instr.sf); write_reg(cpu, instr.rd, val, instr.sf); }

        // ── Logical (register) ──
        Opcode::AndReg | Opcode::OrrReg | Opcode::EorReg | Opcode::AndsReg => exec_logical_reg(cpu, instr),

        // ── Bitfield ──
        Opcode::Sbfm | Opcode::Bfm | Opcode::Ubfm => exec_bitfield(cpu, instr),

        // ── Extended register arithmetic ──
        Opcode::AddExt  => write_reg_sp(cpu, instr.rd, read_base(cpu, instr.rn, instr.sf).wrapping_add(extend_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf)), instr.sf),
        Opcode::SubExt  => write_reg_sp(cpu, instr.rd, read_base(cpu, instr.rn, instr.sf).wrapping_sub(extend_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf)), instr.sf),
        Opcode::AddsExt => { let lhs = read_base(cpu, instr.rn, instr.sf); let rhs = extend_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf); let val = add_flags(cpu, lhs, rhs, instr.sf); if instr.rd != ZERO_REGISTER_INDEX { write_reg_sp(cpu, instr.rd, val, instr.sf); } }
        Opcode::SubsExt => { let lhs = read_base(cpu, instr.rn, instr.sf); let rhs = extend_reg_val(cpu, instr.rm, instr.cond, instr.imm as u8, instr.sf); let val = sub_flags(cpu, lhs, rhs, instr.sf); if instr.rd != ZERO_REGISTER_INDEX { write_reg_sp(cpu, instr.rd, val, instr.sf); } }

        // ── Multiply / divide ──
        Opcode::Madd  => exec_madd(cpu, instr),
        Opcode::Msub  => exec_msub(cpu, instr),
        Opcode::Umulh => { let n = read_reg(cpu, instr.rn, true); let m = read_reg(cpu, instr.rm, true); write_reg(cpu, instr.rd, ((n as u128).wrapping_mul(m as u128) >> 64) as u64, true); }
        Opcode::Smulh => { let n = read_reg(cpu, instr.rn, true) as i64; let m = read_reg(cpu, instr.rm, true) as i64; write_reg(cpu, instr.rd, ((n as i128).wrapping_mul(m as i128) >> 64) as u64, true); }
        Opcode::Udiv  => exec_div(cpu, instr, false),
        Opcode::Sdiv  => exec_div(cpu, instr, true),

        // ── Variable shift ──
        Opcode::Lslv => exec_variable_shift(cpu, instr, ShiftDir::Left),
        Opcode::Lsrv => exec_variable_shift(cpu, instr, ShiftDir::Right),
        Opcode::Asrv => exec_variable_shift(cpu, instr, ShiftDir::ArithRight),
        Opcode::Rorv => exec_variable_shift(cpu, instr, ShiftDir::RotateRight),

        // ── Bit manipulation ──
        Opcode::Rev  => exec_rev(cpu, instr),
        Opcode::Rev32 => { let val = read_reg(cpu, instr.rn, true); let low = (val as u32).swap_bytes() as u64; let high = ((val >> 32) as u32).swap_bytes() as u64; write_reg(cpu, instr.rd, (high << 32) | low, true); }
        Opcode::Rev16 => exec_rev16(cpu, instr),
        Opcode::Rbit => exec_rbit(cpu, instr),
        Opcode::Clz  => exec_clz(cpu, instr),

        // ── System ──
        Opcode::Mrs    => { let val = cpu.sys.read_sys_reg(instr.imm as u16, cpu.pstate.el()); write_reg(cpu, instr.rd, val, true); }
        Opcode::Msr    => exec_msr(cpu, instr),
        Opcode::Tlbi   => { cpu.tlb.invalidate_all(); }
        Opcode::Svc    => return exec_svc(cpu),
        Opcode::Eret   => return exec_eret(cpu),
        Opcode::Brk    => return exec_brk(cpu, bus, instr),
        Opcode::Nop | Opcode::NopBarrier => {
            if instr.cond == 1 {
                let bits = instr.imm as u8;
                if bits & 2 != 0 { cpu.pstate = cpu.pstate.with_irq_masked(true); }
            } else if instr.cond == 2 {
                let bits = instr.imm as u8;
                if bits & 2 != 0 { cpu.pstate = cpu.pstate.with_irq_masked(false); }
            }
        },
        Opcode::Wfi => {
            if cpu.sys.cntp_cval_el0 > cpu.sys.cycle_count {
                cpu.sys.cycle_count = cpu.sys.cntp_cval_el0;
            }
        },
        Opcode::Wfe => {
            if cpu.sys.cntp_cval_el0 > cpu.sys.cycle_count {
                cpu.sys.cycle_count = cpu.sys.cntp_cval_el0;
            }
        },
    }

    advance_pc(cpu);
    check_timer_irq(cpu);
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
//  Post-execution helpers
// ═══════════════════════════════════════════════════════════════════

/// Advance PC by 4 bytes and increment the cycle counter.
fn advance_pc(cpu: &mut Armv8Cpu) {
    cpu.regs.pc += INSTRUCTION_SIZE;
    cpu.sys.cycle_count = cpu.sys.cycle_count.wrapping_add(1);
}

/// Check if the physical timer has expired and deliver an IRQ if so.
fn check_timer_irq(cpu: &mut Armv8Cpu) {
    if cpu.sys.vbar_el1 == 0 { return; }

    let tick_period = TIMER_FREQ_HZ / 1000;
    if cpu.sys.cntp_cval_el0 == 0 || cpu.sys.cntp_cval_el0 > cpu.sys.cycle_count + tick_period * 2 {
        cpu.sys.cntp_cval_el0 = cpu.sys.cycle_count + tick_period;
    }

    if cpu.sys.cycle_count >= cpu.sys.cntp_cval_el0 {
        cpu.sys.irq_pending = true;
        cpu.sys.last_irq_id = TIMER_IRQ_ID;
        cpu.sys.cntp_cval_el0 = cpu.sys.cycle_count + tick_period;
    }

    if cpu.sys.irq_pending && cpu.sys.cycle_count > cpu.sys.cntp_tval_el0 + 100 {
        cpu.sys.cntp_tval_el0 = cpu.sys.cycle_count;
        cpu.sys.irq_pending = false;
        cpu.sys.spsr_el1 = cpu.pstate.with_irq_masked(false).to_u64();
        cpu.sys.elr_el1 = cpu.regs.pc;
        cpu.sys.esr_el1 = 0;

        let pstate_el1 = cpu.pstate.with_el(1).with_irq_masked(true);
        let spsr_bits = pstate_el1.to_u64() | SPSR_M_MASK;
        cpu.pstate = crate::arm64::pstate::ProcessorState::from_u64(spsr_bits);

        cpu.regs.pc = cpu.sys.vbar_el1 + VBAR_IRQ_CURRENT_EL;
    }
}

#[cfg(test)]
mod tests;
