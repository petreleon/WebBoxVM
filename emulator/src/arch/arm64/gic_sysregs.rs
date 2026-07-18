//! GIC CPU-interface system register interception.

use crate::arch::arm64::{Armv8Cpu, Instr, Opcode, read_reg, write_reg};
use crate::constants::*;
use crate::platform::virt::SystemBus;

const ICC_CTLR_EOIMODE: u64 = 1 << 1;

pub(crate) fn handle_gic_sysreg_access(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> bool {
    let core = cpu.core_id as usize;
    match instr.op {
        Opcode::Mrs if instr.imm as u16 == SYSREG_ICC_IAR1_EL1 => {
            let int_id = acknowledge_interrupt(cpu);
            if int_id != GIC_SPURIOUS_INTERRUPT {
                bus.acknowledge_irq_for_cpu(core, int_id as u32);
            }
            write_reg(cpu, instr.rd, int_id, true);
            cpu.regs.pc += INSTRUCTION_SIZE;
            true
        }
        Opcode::Msr if instr.imm as u16 == SYSREG_ICC_EOIR1_EL1 => {
            let int_id = read_reg(cpu, instr.rd, true) as u32;
            cpu.sys.irq_pending = false;
            cpu.sys.last_irq_id = GIC_SPURIOUS_INTERRUPT as u32;
            if cpu.sys.icc_ctlr_el1 & ICC_CTLR_EOIMODE == 0 {
                bus.deactivate_irq_for_cpu(core, int_id);
            }
            cpu.regs.pc += INSTRUCTION_SIZE;
            true
        }
        Opcode::Msr if instr.imm as u16 == SYSREG_ICC_DIR_EL1 => {
            let int_id = read_reg(cpu, instr.rd, true) as u32;
            bus.deactivate_irq_for_cpu(core, int_id);
            cpu.regs.pc += INSTRUCTION_SIZE;
            true
        }
        Opcode::Msr if instr.imm as u16 == SYSREG_ICC_SGI1R_EL1 => {
            let value = read_reg(cpu, instr.rd, true);
            bus.gic.route_sgi1r(core, value);
            cpu.regs.pc += INSTRUCTION_SIZE;
            true
        }
        _ => false,
    }
}

fn acknowledge_interrupt(cpu: &mut Armv8Cpu) -> u64 {
    if cpu.sys.irq_pending {
        cpu.sys.irq_pending = false;
        cpu.sys.last_irq_id as u64
    } else {
        GIC_SPURIOUS_INTERRUPT
    }
}

#[cfg(test)]
mod tests;
