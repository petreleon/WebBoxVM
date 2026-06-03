//! GIC CPU-interface system register interception.

use crate::arm64::{Armv8Cpu, Instr, Opcode, read_reg, write_reg};
use crate::bus::SystemBus;
use crate::constants::*;

pub(crate) fn handle_gic_sysreg_access(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> bool {
    match instr.op {
        Opcode::Mrs if instr.imm as u16 == SYSREG_ICC_IAR1_EL1 => {
            let int_id = acknowledge_interrupt(cpu);
            if int_id != GIC_SPURIOUS_INTERRUPT {
                bus.gic.clear_pending(int_id as u32);
            }
            write_reg(cpu, instr.rd, int_id, true);
            cpu.regs.pc += INSTRUCTION_SIZE;
            true
        }
        Opcode::Msr if instr.imm as u16 == SYSREG_ICC_EOIR1_EL1 => {
            let int_id = read_reg(cpu, instr.rd, true) as u32;
            cpu.sys.irq_pending = false;
            cpu.sys.last_irq_id = GIC_SPURIOUS_INTERRUPT as u32;
            bus.gic.clear_pending(int_id);
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
