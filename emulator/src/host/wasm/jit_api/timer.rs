use crate::arch::arm64::Armv8Cpu;
use crate::constants::{
    PHYSICAL_TIMER_IRQ_ID, VBAR_IRQ_CURRENT_EL, VBAR_IRQ_LOWER_EL_AARCH64, VIRTUAL_TIMER_IRQ_ID,
};

pub(super) fn deliver_jit_timer_boundary(cpu: &mut Armv8Cpu) {
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
    }
}
