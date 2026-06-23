use crate::constants::{PL011_UART_IRQ_ID, VBAR_IRQ_CURRENT_EL};
use crate::runtime::Machine;

#[test]
fn deliver_irq_fast_path_ignores_idle_external_poll() {
    let mut machine = Machine::new(1);
    let cpu = machine.core_mut(0);
    cpu.regs.pc = 0x4000_0000;
    cpu.sys.vbar_el1 = 0x8000_0000;
    cpu.pstate = cpu.pstate.with_el(1).with_irq_masked(false);

    machine.deliver_irq(0);

    let cpu = machine.core(0);
    assert_eq!(cpu.regs.pc, 0x4000_0000);
    assert!(!cpu.sys.irq_pending);
}

#[test]
fn deliver_irq_takes_pending_enabled_external_irq() {
    let mut machine = Machine::new(1);
    let cpu = machine.core_mut(0);
    cpu.regs.pc = 0x4000_0000;
    cpu.sys.vbar_el1 = 0x8000_0000;
    cpu.pstate = cpu.pstate.with_el(1).with_irq_masked(false);

    machine.bus.gic.enable_interrupt(PL011_UART_IRQ_ID);
    machine.bus.set_irq_pending(PL011_UART_IRQ_ID);
    machine.deliver_irq(0);

    let cpu = machine.core(0);
    assert_eq!(cpu.regs.pc, cpu.sys.vbar_el1 + VBAR_IRQ_CURRENT_EL);
    assert_eq!(cpu.sys.last_irq_id, PL011_UART_IRQ_ID);
    assert!(cpu.sys.irq_pending);
}
