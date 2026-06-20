//! Instruction execution engine — mutates CPU and bus state for every ARM64 instruction.
//!
//! For each decoded instruction, this module:
//!   1. Reads source registers (handling XZR/WZR semantics)
//!   2. Performs the operation (ALU, load/store, branch, etc.)
//!   3. Writes the result to the destination register
//!   4. Increments PC and the cycle counter
//!   5. Checks for timer interrupt delivery

mod alu;
mod branch;
mod dispatch;
mod load_store;
mod sve;
mod system;

pub(super) use super::opcodes::{Instr, Opcode};
use alu::*;
use branch::{branch, branch_link, branch_link_reg, branch_reg, branch_target};
use load_store::{
    exec_atomic, exec_exclusive, exec_ldp_stp, exec_ldr_lit, exec_ldr_str, exec_mops, exec_mte_gpr,
    exec_mte_mem,
};
use sve::*;
use system::{exec_brk, exec_dc_gva, exec_dc_zva, exec_eret, exec_msr, exec_svc, exec_udf};

use super::Armv8Cpu;
use super::helpers::{cond_taken, read_base, read_reg, write_reg, write_reg_sp};
use crate::constants::*;
use crate::platform::virt::SystemBus;
use std::env;

/// Execute one decoded instruction, returning an error string if something goes wrong.
pub fn execute(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    match dispatch::execute_body(cpu, bus, instr)? {
        dispatch::Flow::Advance => {
            advance_pc(cpu);
            check_timer_irq(cpu);
            Ok(())
        }
        dispatch::Flow::Return => Ok(()),
    }
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
    if cpu.sys.vbar_el1 == 0 {
        return;
    }

    if cpu.sys.cntv_expired() && cpu.sys.cntv_unmasked() {
        cpu.sys.irq_pending = true;
        cpu.sys.last_irq_id = VIRTUAL_TIMER_IRQ_ID;
    } else if cpu.sys.cntp_expired() && cpu.sys.cntp_unmasked() {
        cpu.sys.irq_pending = true;
        cpu.sys.last_irq_id = PHYSICAL_TIMER_IRQ_ID;
    }

    if cpu.sys.irq_pending && !cpu.pstate.irq_masked() {
        trace_daif(cpu, "irq exception");
        cpu.clear_exclusive();
        let from_lower_el = cpu.pstate.el() == 0;
        cpu.sys.spsr_el1 = cpu.pstate.to_u64();
        cpu.sys.elr_el1 = cpu.regs.pc;
        cpu.sys.esr_el1 = 0;

        cpu.enter_el1_exception(from_lower_el);
        cpu.regs.pc = cpu.sys.vbar_el1
            + if from_lower_el {
                VBAR_IRQ_LOWER_EL_AARCH64
            } else {
                VBAR_IRQ_CURRENT_EL
            };
        trace_daif(cpu, "irq exception ->");
    }
}

fn trace_daif(cpu: &Armv8Cpu, label: &str) {
    if env::var_os("WEBBOXVM_TRACE_DAIF").is_some() {
        eprintln!(
            "DAIF {label} pc=0x{:016x} pstate=0x{:x} irq_masked={}",
            cpu.regs.pc,
            cpu.pstate.to_u64(),
            cpu.pstate.irq_masked()
        );
    }
}

#[cfg(test)]
mod tests;
