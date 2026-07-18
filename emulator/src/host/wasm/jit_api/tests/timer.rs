use super::super::commit::{can_commit_jit_block_now, commit_jit_state};
use crate::arch::arm64::jit::WasmJitCpuState;
use crate::constants::{
    PL011_UART_IRQ_ID, RAM_BASE, TIMER_CTL_ENABLE, UART_BASE, UART_IMSC_OFFSET,
    VBAR_IRQ_LOWER_EL_AARCH64, VIRTUAL_TIMER_IRQ_ID,
};
use crate::runtime::Machine;

const UART_RX_IRQ_MASK: u64 = (1 << 4) | (1 << 6);

#[test]
fn commit_accepts_block_ending_on_timer_deadline() {
    let mut machine = Machine::new(1);
    let cpu = &mut machine.cpus[0];
    cpu.regs.pc = RAM_BASE;
    cpu.sys.vbar_el1 = RAM_BASE + 0x8000;
    cpu.sys.cycle_count = 100;
    cpu.sys.cntv_ctl_el0 = TIMER_CTL_ENABLE;
    cpu.sys.cntv_cval_el0 = 104;
    cpu.pstate = cpu.pstate.with_el(0).with_irq_masked(false);
    let mut state = WasmJitCpuState::from_cpu(cpu);
    state.pc = RAM_BASE + 16;

    commit_jit_state(&state, &mut machine, 0, 4, RAM_BASE + 16)
        .expect("exact timer boundary should commit");

    let cpu = &machine.cpus[0];
    assert_eq!(cpu.sys.cycle_count, 104);
    assert_eq!(machine.virtual_time, 104);
    assert_eq!(cpu.sys.last_irq_id, VIRTUAL_TIMER_IRQ_ID);
    assert_eq!(cpu.sys.elr_el1, RAM_BASE + 16);
    assert_eq!(cpu.regs.pc, RAM_BASE + 0x8000 + VBAR_IRQ_LOWER_EL_AARCH64);
}

#[test]
fn commit_still_rejects_block_crossing_timer_deadline() {
    let mut machine = Machine::new(1);
    let cpu = &mut machine.cpus[0];
    cpu.regs.pc = RAM_BASE;
    cpu.sys.cycle_count = 100;
    cpu.sys.cntv_ctl_el0 = TIMER_CTL_ENABLE;
    cpu.sys.cntv_cval_el0 = 103;
    let mut state = WasmJitCpuState::from_cpu(cpu);
    state.pc = RAM_BASE + 16;

    let err = commit_jit_state(&state, &mut machine, 0, 4, RAM_BASE + 16)
        .expect_err("mid-block timer deadline must stay in interpreter");

    assert!(err.contains("timer deadline"), "{err}");
}

#[test]
fn preflight_rejects_block_crossing_timer_deadline() {
    let mut machine = Machine::new(1);
    let cpu = &mut machine.cpus[0];
    cpu.sys.cycle_count = 100;
    cpu.sys.cntv_ctl_el0 = TIMER_CTL_ENABLE;
    cpu.sys.cntv_cval_el0 = 103;

    let err = can_commit_jit_block_now(&mut machine, 0, 4)
        .expect_err("preflight should catch mid-block timer deadlines");

    assert!(err.contains("timer deadline"), "{err}");
    assert_eq!(machine.cpus[0].sys.cycle_count, 100);
}

#[test]
fn preflight_accepts_block_ending_on_timer_deadline() {
    let mut machine = Machine::new(1);
    let cpu = &mut machine.cpus[0];
    cpu.sys.cycle_count = 100;
    cpu.sys.cntv_ctl_el0 = TIMER_CTL_ENABLE;
    cpu.sys.cntv_cval_el0 = 104;

    can_commit_jit_block_now(&mut machine, 0, 4)
        .expect("exact timer boundary is still committable");
}

#[test]
fn preflight_skips_interrupt_refresh_when_irqs_are_masked() {
    let mut machine = Machine::new(1);
    machine.cpus[0].pstate = machine.cpus[0].pstate.with_irq_masked(true);
    machine.bus.gic.enable_interrupt(PL011_UART_IRQ_ID);
    machine
        .bus
        .uart
        .write(UART_BASE + UART_IMSC_OFFSET, 4, UART_RX_IRQ_MASK);
    machine.bus.feed_uart_input("a");
    machine.bus.clear_irq_pending(PL011_UART_IRQ_ID);

    can_commit_jit_block_now(&mut machine, 0, 4).expect("masked IRQs cannot block JIT commit");

    assert_eq!(machine.bus.gic.next_pending_enabled(), None);

    machine.cpus[0].pstate = machine.cpus[0].pstate.with_irq_masked(false);
    let err = can_commit_jit_block_now(&mut machine, 0, 4)
        .expect_err("unmasked IRQs must still block JIT commit");

    assert!(err.contains("unmasked pending IRQ"), "{err}");
    assert_eq!(
        machine.bus.gic.next_pending_enabled(),
        Some(PL011_UART_IRQ_ID)
    );
}
