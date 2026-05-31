//! System instruction execution: MSR, SVC, ERET, BRK.

use super::Instr;
use crate::arm64::helpers::read_reg;
use crate::arm64::mmu::{Fault, translate_write};
use crate::arm64::{Armv8Cpu, ProcessorState};
use crate::bus::SystemBus;
use crate::constants::*;
use std::env;

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

pub(super) fn exec_svc(cpu: &mut Armv8Cpu, imm16: u64) -> Result<(), &'static str> {
    let from_lower_el = cpu.pstate.el() == 0;
    if from_lower_el && env::var_os("WEBBOXVM_TRACE_SYSCALLS").is_some() {
        eprintln!(
            "SVC pc={:#018x} nr={} x0={:#018x} x1={:#018x} x2={:#018x} x3={:#018x} x4={:#018x} x5={:#018x} sp={:#018x}",
            cpu.regs.pc,
            cpu.regs.x(8),
            cpu.regs.x(0),
            cpu.regs.x(1),
            cpu.regs.x(2),
            cpu.regs.x(3),
            cpu.regs.x(4),
            cpu.regs.x(5),
            cpu.regs.sp,
        );
    }
    cpu.clear_exclusive();
    cpu.sys.elr_el1 = cpu.regs.pc + INSTRUCTION_SIZE;
    cpu.sys.spsr_el1 = cpu.pstate.to_u64();
    cpu.sys.esr_el1 = (ESR_EC_SVC64 << 26) | (imm16 & 0xffff);
    cpu.enter_el1_exception(from_lower_el);
    if from_lower_el && env::var_os("WEBBOXVM_TRACE_SYSCALL_FRAME").is_some() {
        cpu.trace_syscall_stack_top = cpu.regs.sp;
        cpu.trace_syscall_access_budget = 512;
    }
    cpu.regs.pc = cpu.sys.vbar_el1
        + if from_lower_el {
            VBAR_SYNC_LOWER_EL_AARCH64
        } else {
            VBAR_SYNC_CURRENT_EL
        };
    Ok(())
}

pub(super) fn exec_eret(cpu: &mut Armv8Cpu) -> Result<(), &'static str> {
    trace_daif(cpu, "eret");
    let target = ProcessorState::from_u64(cpu.sys.spsr_el1);
    if target.el() == 0 && env::var_os("WEBBOXVM_TRACE_CPACR").is_some() {
        eprintln!(
            "CPACR eret-to-el0 pc={:#018x} cpacr={:#018x} fpen={}",
            cpu.sys.elr_el1,
            cpu.sys.cpacr_el1,
            (cpu.sys.cpacr_el1 & CPACR_FPEN_MASK) >> CPACR_FPEN_SHIFT,
        );
    }
    if target.el() == 0 && env::var_os("WEBBOXVM_TRACE_SYSCALLS").is_some() {
        eprintln!(
            "ERET to EL0 pc={:#018x} x0={:#018x} x8={:#018x} spsr={:#x} sp={:#018x} sp_el0={:#018x} sp_el1={:#018x}",
            cpu.sys.elr_el1,
            cpu.regs.x(0),
            cpu.regs.x(8),
            cpu.sys.spsr_el1,
            cpu.regs.sp,
            cpu.sys.sp_el0,
            cpu.sys.sp_el1,
        );
    }
    cpu.regs.pc = cpu.sys.elr_el1;
    cpu.eret_to(target);
    trace_daif(cpu, "eret ->");
    Ok(())
}

pub(super) fn exec_brk(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    let el = cpu.pstate.el();
    let imm16 = instr.imm;
    let pc = cpu.regs.pc;
    cpu.clear_exclusive();

    if env::var_os("WEBBOXVM_TRACE_BRK").is_some() {
        eprintln!("BRK pc={pc:#018x} el={el} imm16=0x{imm16:x}");
        super::debug::dump_instructions("PC", pc, cpu, bus);
        super::debug::dump_instructions("LR", cpu.regs.x(LINK_REGISTER_INDEX), cpu, bus);
        super::debug::dump_string_pointers(cpu, bus);
        super::debug::dump_stack(cpu, bus);
    }

    let from_lower_el = el == 0;
    let esr = (ESR_EC_BRK64 << 26) | (imm16 & 0xffff);
    cpu.sys.elr_el1 = pc;
    cpu.sys.spsr_el1 = cpu.pstate.to_u64();
    cpu.sys.esr_el1 = esr;

    cpu.enter_el1_exception(from_lower_el);
    cpu.regs.pc = cpu.sys.vbar_el1
        + if from_lower_el {
            VBAR_SYNC_LOWER_EL_AARCH64
        } else {
            VBAR_SYNC_CURRENT_EL
        };
    Ok(())
}

fn fault_to_error(fault: Fault) -> &'static str {
    match fault {
        Fault::TranslationFault => "translation fault",
        Fault::AccessFlagFault => "access flag fault",
        Fault::PermissionFault => "permission fault",
    }
}

fn trace_daif(cpu: &Armv8Cpu, label: &str) {
    if env::var_os("WEBBOXVM_TRACE_DAIF").is_some() {
        eprintln!(
            "DAIF {label} pc=0x{:016x} pstate=0x{:x} spsr=0x{:x} elr=0x{:016x} irq_masked={}",
            cpu.regs.pc,
            cpu.pstate.to_u64(),
            cpu.sys.spsr_el1,
            cpu.sys.elr_el1,
            cpu.pstate.irq_masked()
        );
    }
}
