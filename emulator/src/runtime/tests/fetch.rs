use crate::constants::{KERNEL_VA_BASE, RAM_BASE, SCTLR_MMU_ENABLE, VBAR_SYNC_CURRENT_EL};
use crate::runtime::Machine;

#[test]
fn translate_fetch_uses_identity_when_mmu_disabled() {
    let mut machine = Machine::new(1);
    let pc = RAM_BASE + 0x1234;
    let cpu = machine.core_mut(0);
    cpu.sys.sctlr_el1 = 0;
    cpu.sys.vbar_el1 = KERNEL_VA_BASE;
    cpu.sys.ttbr1_el1 = 0xDEAD_0000;

    assert_eq!(machine.translate_fetch(0, pc, 1), Some(pc));
    assert_eq!(machine.fetch_faults, 0);
    assert_eq!(machine.total_steps, 0);
}

#[test]
fn translate_fetch_still_faults_when_mmu_enabled() {
    let mut machine = Machine::new(1);
    let pc = KERNEL_VA_BASE + 0x4000;
    let cpu = machine.core_mut(0);
    cpu.pstate = cpu.pstate.with_el(1);
    cpu.sys.sctlr_el1 = SCTLR_MMU_ENABLE;
    cpu.sys.vbar_el1 = KERNEL_VA_BASE + 0x1000;

    assert_eq!(machine.translate_fetch(0, pc, 1), None);

    let cpu = machine.core(0);
    assert_eq!(machine.fetch_faults, 1);
    assert_eq!(machine.total_steps, 1);
    assert_eq!(cpu.regs.pc, cpu.sys.vbar_el1 + VBAR_SYNC_CURRENT_EL);
    assert_eq!(cpu.sys.far_el1, pc);
}
