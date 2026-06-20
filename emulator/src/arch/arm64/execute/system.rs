//! System instruction execution: MSR, DC ZVA, and exception helpers.
use super::Instr;
use crate::arch::arm64::Armv8Cpu;
use crate::arch::arm64::helpers::read_reg;
use crate::arch::arm64::mmu::Fault;
use crate::constants::*;
use std::env;

mod cache;
mod exceptions;
mod helpers;

pub(super) use cache::{exec_dc_gva, exec_dc_zva};
pub(super) use exceptions::{exec_brk, exec_eret, exec_svc, exec_udf};
use helpers::trace_daif;

pub(super) fn exec_msr(cpu: &mut Armv8Cpu, instr: Instr) {
    let val = read_reg(cpu, instr.rd, true);
    let sysreg_id = instr.imm as u16;
    let old_cpacr = cpu.sys.cpacr_el1;
    cpu.sys.write_sys_reg(sysreg_id, val);
    match sysreg_id {
        SYSREG_TTBR0_EL1 | SYSREG_TTBR1_EL1 | SYSREG_TCR_EL1 => cpu.tlb.invalidate_all(),
        SYSREG_CPACR_EL1 => {
            if env::var_os("WEBBOXVM_TRACE_CPACR").is_some() {
                eprintln!(
                    "CPACR msr pc=0x{:016x} old=0x{old_cpacr:016x} new=0x{:016x} fpen={}",
                    cpu.regs.pc,
                    cpu.sys.cpacr_el1,
                    (cpu.sys.cpacr_el1 & CPACR_FPEN_MASK) >> CPACR_FPEN_SHIFT,
                );
            }
        }
        SYSREG_DAIF => {
            trace_daif(cpu, "msr daif");
            cpu.pstate = cpu.pstate.with_daif(val);
            trace_daif(cpu, "msr daif ->");
        }
        _ => {}
    }
}
