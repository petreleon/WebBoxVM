//! System instruction execution: MSR, DC ZVA, and exception helpers.
use super::Instr;
use crate::arm64::Armv8Cpu;
use crate::arm64::helpers::read_reg;
use crate::arm64::mmu::{Fault, translate_write};
use crate::bus::SystemBus;
use crate::constants::*;
use std::env;

mod exceptions;
mod helpers;

pub(super) use exceptions::{exec_brk, exec_eret, exec_svc, exec_udf};
use helpers::{fault_to_error, trace_daif};

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

pub(super) fn exec_dc_zva(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    if DCZID_EL0_VAL & 0x10 != 0 {
        return Ok(());
    }

    let block_size = 4u64 << (DCZID_EL0_VAL & 0xF);
    let base = read_reg(cpu, instr.rd, true) & !(block_size - 1);
    let mut offset = 0;
    while offset < block_size {
        let size = (block_size - offset).min(8) as u8;
        let va = base + offset;
        let pa = translate_write(&cpu.sys, &mut bus.mem, va, cpu.pstate.el()).map_err(|fault| {
            cpu.sys.far_el1 = va;
            fault_to_error(fault)
        })?;
        bus.write(pa, size, 0);
        offset += size as u64;
    }
    Ok(())
}
