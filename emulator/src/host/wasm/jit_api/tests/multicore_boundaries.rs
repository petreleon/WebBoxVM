use super::super::commit_boundary::can_commit_jit_block_now;
use crate::arch::arm64::CpuLifecycle;
use crate::constants::{RAM_BASE, TIMER_CTL_ENABLE};
use crate::runtime::Machine;

#[test]
fn jit_preflight_does_not_skip_a_waiting_cores_timer_deadline() {
    let mut machine = Machine::new(2);
    machine.virtual_time = 10;
    assert_eq!(machine.prepare_next_core(), Some(0));
    let waiting = &mut machine.cpus[1];
    waiting.lifecycle = CpuLifecycle::WaitingForInterrupt;
    waiting.sys.vbar_el1 = RAM_BASE + 0x8000;
    waiting.sys.cntv_ctl_el0 = TIMER_CTL_ENABLE;
    waiting.sys.cntv_cval_el0 = 12;

    can_commit_jit_block_now(&mut machine, 0, 2)
        .expect("a block may end exactly at another core's wake deadline");
    let err = can_commit_jit_block_now(&mut machine, 0, 3)
        .expect_err("a block must not skip over another core's wake deadline");
    assert!(err.contains("core 1 remote timer deadline"), "{err}");
}

#[test]
fn jit_preflight_does_not_skip_a_runnable_cores_timer_deadline() {
    let mut machine = Machine::new(2);
    machine.virtual_time = 10;
    assert_eq!(machine.prepare_next_core(), Some(0));
    let remote = &mut machine.cpus[1];
    remote.lifecycle = CpuLifecycle::Runnable;
    remote.sys.cntv_ctl_el0 = TIMER_CTL_ENABLE;
    remote.sys.cntv_cval_el0 = 12;

    can_commit_jit_block_now(&mut machine, 0, 2)
        .expect("a block may end exactly at another runnable core's timer deadline");
    let err = can_commit_jit_block_now(&mut machine, 0, 3)
        .expect_err("a block must not skip a runnable core's timer deadline");
    assert!(err.contains("core 1 remote timer deadline"), "{err}");
}

#[test]
fn multicore_jit_preflight_rejects_an_active_core_with_stale_time() {
    let mut machine = Machine::new(2);
    machine.virtual_time = 20;
    machine.active_core = 1;
    machine.cpus[1].lifecycle = CpuLifecycle::Runnable;
    machine.cpus[1].sys.cycle_count = 7;

    let err = can_commit_jit_block_now(&mut machine, 1, 1)
        .expect_err("an active core must still be prepared by the scheduler");

    assert!(err.contains("core 1 was not scheduler-prepared"), "{err}");
    assert!(err.contains("core cycle=7 virtual time=20"), "{err}");
}
