use super::*;

const TLBI_VMALLE1IS: u64 = 0xd508_831f;
const YIELD: u64 = 0xd503_203f;

#[test]
fn yield_does_not_fast_forward_global_time_while_another_cpu_is_runnable() {
    let mut machine = Machine::new(2);
    let code = RAM_BASE + 0x5800;
    machine.bus.mem.write(code, 4, YIELD);
    machine.cpus[0].regs.pc = code;
    machine.cpus[0].sys.cntv_ctl_el0 = TIMER_CTL_ENABLE;
    machine.cpus[0].sys.cntv_cval_el0 = 10_000;
    machine.cpus[1].lifecycle = CpuLifecycle::Runnable;

    assert_eq!(machine.run(1), 1);

    assert_eq!(machine.virtual_time, 1);
    assert_eq!(machine.cpus[0].sys.cycle_count, 1);
    assert_eq!(machine.active_core, 1);
}

#[test]
fn tlbi_broadcast_invalidates_a_remote_cpu_ancestor_mapping() {
    let mut machine = Machine::new(2);
    let code = RAM_BASE + 0x6000;
    let l1 = RAM_BASE + 0x1_0000;
    let old_l2 = RAM_BASE + 0x1_1000;
    let old_l3 = RAM_BASE + 0x1_2000;
    let new_l2 = RAM_BASE + 0x1_3000;
    let new_l3 = RAM_BASE + 0x1_4000;
    let old_page = RAM_BASE + 0x2_0000;
    let new_page = RAM_BASE + 0x2_1000;
    let va = 0xffff_ff80_0000_0000;

    machine.bus.mem.write(code, 4, TLBI_VMALLE1IS);
    machine.bus.mem.write(l1, 8, old_l2 | 0b11);
    machine.bus.mem.write(old_l2, 8, old_l3 | 0b11);
    machine.bus.mem.write(old_l3, 8, old_page | 0b01);
    machine.bus.mem.write(new_l2, 8, new_l3 | 0b11);
    machine.bus.mem.write(new_l3, 8, new_page | 0b01);

    let secondary = &mut machine.cpus[1];
    secondary.sys.ttbr1_el1 = l1;
    secondary.sys.tcr_el1 = (25 << 16) | 25;
    secondary.sys.sctlr_el1 = SCTLR_MMU_ENABLE;
    assert_eq!(
        translate(&secondary.sys, &mut secondary.tlb, &machine.bus.mem, va).unwrap(),
        old_page
    );

    machine.bus.mem.write(l1, 8, new_l2 | 0b11);
    machine.cpus[0].regs.pc = code;
    machine.cpus[1].lifecycle = CpuLifecycle::Runnable;

    assert_eq!(machine.run(1), 1);
    let secondary = &mut machine.cpus[1];
    assert_eq!(
        translate(&secondary.sys, &mut secondary.tlb, &machine.bus.mem, va).unwrap(),
        new_page
    );
}
