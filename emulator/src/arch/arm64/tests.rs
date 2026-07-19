use super::*;
use crate::constants::{MPIDR_RES1, SYSREG_MPIDR_EL1};

#[test]
fn boot_state_is_runnable_on_core_zero() {
    let cpu = Armv8Cpu::new();
    assert_eq!(cpu.core_id, 0);
    assert_eq!(cpu.lifecycle, CpuLifecycle::Runnable);
    assert_eq!(cpu.pstate.el(), 3);
    assert_eq!(cpu.regs.x(0), 0);
    assert_eq!(cpu.sys.sctlr_el1, 0);
}

#[test]
fn reset_clears_architecture_but_preserves_identity_and_lifecycle() {
    let mut cpu = Armv8Cpu::with_core(7);
    cpu.lifecycle = CpuLifecycle::PoweredOff;
    cpu.event_register = true;
    cpu.waiting_for_event = true;
    cpu.regs.set_x(0, 42);
    cpu.sys.sctlr_el1 = 1;

    cpu.reset();

    assert_eq!(cpu.core_id, 7);
    assert_eq!(cpu.lifecycle, CpuLifecycle::PoweredOff);
    assert!(!cpu.event_register);
    assert!(!cpu.waiting_for_event);
    assert_eq!(cpu.regs.x(0), 0);
    assert_eq!(cpu.sys.sctlr_el1, 0);
    assert_eq!(cpu.sys.mpidr_el1, MPIDR_RES1 | 7);
}

#[test]
fn mpidr_uses_core_id_as_affinity_zero() {
    let mut primary = Armv8Cpu::default();
    let mut secondary = Armv8Cpu::with_core(3);

    assert_eq!(primary.sys.read_sys_reg(SYSREG_MPIDR_EL1, 1), MPIDR_RES1);
    assert_eq!(
        secondary.sys.read_sys_reg(SYSREG_MPIDR_EL1, 1),
        MPIDR_RES1 | 3
    );
}

#[test]
fn range_clear_drops_only_overlapping_exclusive_reservation() {
    let mut cpu = Armv8Cpu::new();
    cpu.reserve_exclusive(0x1000, 8);

    cpu.clear_exclusive_range_if_overlaps(0x2000, 0x100);
    assert!(cpu.exclusive_matches(0x1000, 8));

    cpu.clear_exclusive_range_if_overlaps(0x1004, 0x100);
    assert!(cpu.exclusive.is_none());
}

#[test]
fn range_clear_uses_the_advertised_exclusive_reservation_granule() {
    let mut cpu = Armv8Cpu::new();
    cpu.reserve_exclusive(0x1020, 8);

    cpu.clear_exclusive_range_if_overlaps(0x1000, 8);

    assert!(cpu.exclusive.is_none());
}

#[test]
fn lower_el_exception_selects_el1_stack_and_saves_el0_stack() {
    let mut cpu = Armv8Cpu::new();
    cpu.pstate = ProcessorState::new()
        .with_el(0)
        .with_sp_select(false)
        .with_irq_masked(false);
    cpu.regs.sp = 0x1000;
    cpu.sys.sp_el1 = 0x2000;

    cpu.enter_el1_exception(true);

    assert_eq!(cpu.sys.sp_el0, 0x1000);
    assert_eq!(cpu.regs.sp, 0x2000);
    assert_eq!(cpu.pstate.el(), 1);
    assert!(cpu.pstate.sp_select());
    assert!(cpu.pstate.all_exceptions_masked());
}
