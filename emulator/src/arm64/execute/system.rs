//! System instruction execution: MSR, SVC, ERET, BRK.

use crate::arm64::helpers::read_reg;
use crate::arm64::mmu::translate;
use crate::arm64::Tlb;
use crate::arm64::Armv8Cpu;
use crate::bus::SystemBus;
use crate::constants::*;
use super::Instr;

pub(super) fn exec_msr(cpu: &mut Armv8Cpu, instr: Instr) {
    let val = read_reg(cpu, instr.rd, true);
    let sysreg_id = instr.imm as u16;
    cpu.sys.write_sys_reg(sysreg_id, val);
    match sysreg_id {
        SYSREG_TTBR0_EL1 | SYSREG_TTBR1_EL1 | SYSREG_TCR_EL1 => cpu.tlb.invalidate_all(),
        SYSREG_DAIF => {
            cpu.pstate = cpu.pstate.with_irq_masked((val >> 7) & 1 != 0);
        }
        _ => {}
    }
}

pub(super) fn exec_svc(cpu: &mut Armv8Cpu) -> Result<(), &'static str> {
    cpu.sys.elr_el1 = cpu.regs.pc + INSTRUCTION_SIZE;
    cpu.sys.spsr_el1 = cpu.pstate.to_u64();
    cpu.pstate = cpu.pstate.with_el(1);
    cpu.regs.pc = cpu.sys.vbar_el1 + VBAR_SYNC_LOWER_EL_AARCH64;
    Ok(())
}

pub(super) fn exec_eret(cpu: &mut Armv8Cpu) -> Result<(), &'static str> {
    cpu.regs.pc = cpu.sys.elr_el1;
    cpu.pstate = crate::arm64::pstate::ProcessorState::from_u64(cpu.sys.spsr_el1);
    Ok(())
}

pub(super) fn exec_brk(cpu: &mut Armv8Cpu, bus: &SystemBus, instr: Instr) -> Result<(), &'static str> {
    let el = cpu.pstate.el();
    let imm16 = instr.imm;
    let pc = cpu.regs.pc;

    let pa_str = match translate(&cpu.sys, &mut Tlb::new(), &bus.mem, pc) {
        Ok(pa) => format!("PA=0x{:016x}", pa),
        Err(_) => "PA=UNMAPPED".to_string(),
    };

    eprintln!("--- BRK HIT ---");
    eprintln!("  PC={:#018x}  {}  EL={}  imm16=0x{:x}", pc, pa_str, el, imm16);
    eprintln!("  X0={:#018x}  X1={:#018x}  X2={:#018x}  X3={:#018x}", cpu.regs.x(0), cpu.regs.x(1), cpu.regs.x(2), cpu.regs.x(3));
    eprintln!("  X4={:#018x}  X5={:#018x}  X6={:#018x}  X7={:#018x}", cpu.regs.x(4), cpu.regs.x(5), cpu.regs.x(6), cpu.regs.x(7));
    eprintln!("  X19={:#018x}  X20={:#018x}  X21={:#018x}  X29={:#018x}  LR={:#018x}  SP={:#018x}", cpu.regs.x(19), cpu.regs.x(20), cpu.regs.x(21), cpu.regs.x(29), cpu.regs.x(30), cpu.regs.sp);
    eprintln!("  VBAR_EL1={:#018x}  ELR_EL1={:#018x}  SPSR_EL1={:#018x}", cpu.sys.vbar_el1, cpu.sys.elr_el1, cpu.sys.spsr_el1);

    super::debug::dump_instructions("PC", pc, cpu, bus);
    super::debug::dump_instructions("LR", cpu.regs.x(LINK_REGISTER_INDEX), cpu, bus);
    super::debug::dump_string_pointers(cpu, bus);
    super::debug::dump_stack(cpu, bus);

    let esr = (0x3Cu64 << 26) | (imm16 & 0xffff);
    cpu.sys.elr_el1 = pc;
    cpu.sys.spsr_el1 = cpu.pstate.to_u64();
    cpu.sys.esr_el1 = esr;

    let pstate_el1 = cpu.pstate.with_el(1);
    let spsr_bits = pstate_el1.to_u64() | SPSR_M_MASK;
    cpu.pstate = crate::arm64::pstate::ProcessorState::from_u64(spsr_bits);

    cpu.regs.pc = cpu.sys.vbar_el1 + VBAR_SYNC_CURRENT_EL;
    Ok(())
}
