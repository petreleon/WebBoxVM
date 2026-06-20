use crate::arch::arm64::{Armv8Cpu, Instr, ProcessorState};
use crate::constants::*;
use crate::observability::dump_breakpoint_context;
use crate::platform::virt::SystemBus;
use std::env;

pub(in crate::arch::arm64::execute) fn exec_svc(
    cpu: &mut Armv8Cpu,
    imm16: u64,
) -> Result<(), &'static str> {
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
    take_el1_sync_vector(cpu, from_lower_el);
    Ok(())
}

pub(in crate::arch::arm64::execute) fn exec_eret(cpu: &mut Armv8Cpu) -> Result<(), &'static str> {
    super::trace_daif(cpu, "eret");
    let target = ProcessorState::from_u64(cpu.sys.spsr_el1);
    trace_eret(cpu, target.el());
    cpu.regs.pc = cpu.sys.elr_el1;
    cpu.eret_to(target);
    super::trace_daif(cpu, "eret ->");
    Ok(())
}

pub(in crate::arch::arm64::execute) fn exec_brk(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    let el = cpu.pstate.el();
    let imm16 = instr.imm;
    let pc = cpu.regs.pc;
    cpu.clear_exclusive();
    trace_brk(cpu, bus, pc, el, imm16);
    let from_lower_el = el == 0;
    cpu.sys.elr_el1 = pc;
    cpu.sys.spsr_el1 = cpu.pstate.to_u64();
    cpu.sys.esr_el1 = (ESR_EC_BRK64 << 26) | (imm16 & 0xffff);
    cpu.enter_el1_exception(from_lower_el);
    take_el1_sync_vector(cpu, from_lower_el);
    Ok(())
}

pub(in crate::arch::arm64::execute) fn exec_udf(cpu: &mut Armv8Cpu) -> Result<(), &'static str> {
    let from_lower_el = cpu.pstate.el() == 0;
    cpu.clear_exclusive();
    cpu.sys.elr_el1 = cpu.regs.pc;
    cpu.sys.spsr_el1 = cpu.pstate.to_u64();
    cpu.sys.esr_el1 = (ESR_EC_UNKNOWN << 26) | ESR_IL;
    cpu.enter_el1_exception(from_lower_el);
    take_el1_sync_vector(cpu, from_lower_el);
    Ok(())
}

fn take_el1_sync_vector(cpu: &mut Armv8Cpu, from_lower_el: bool) {
    cpu.regs.pc = cpu.sys.vbar_el1
        + if from_lower_el {
            VBAR_SYNC_LOWER_EL_AARCH64
        } else {
            VBAR_SYNC_CURRENT_EL
        };
}

fn trace_eret(cpu: &Armv8Cpu, target_el: u8) {
    if target_el == 0 && env::var_os("WEBBOXVM_TRACE_CPACR").is_some() {
        eprintln!(
            "CPACR eret-to-el0 pc={:#018x} cpacr={:#018x} fpen={}",
            cpu.sys.elr_el1,
            cpu.sys.cpacr_el1,
            (cpu.sys.cpacr_el1 & CPACR_FPEN_MASK) >> CPACR_FPEN_SHIFT,
        );
    }
    if target_el == 0 && env::var_os("WEBBOXVM_TRACE_SYSCALLS").is_some() {
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
}

fn trace_brk(cpu: &Armv8Cpu, bus: &SystemBus, pc: u64, el: u8, imm16: u64) {
    if env::var_os("WEBBOXVM_TRACE_BRK").is_none() {
        return;
    }
    eprintln!("BRK pc={pc:#018x} el={el} imm16=0x{imm16:x}");
    dump_breakpoint_context(cpu, bus, pc);
}
