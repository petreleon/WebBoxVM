use super::super::commit::commit_jit_state;
use super::super::commit_boundary::can_commit_jit_block_now;
use super::super::prepared_commit::commit_finished_jit_state;
use crate::arch::arm64::CpuLifecycle;
use crate::arch::arm64::jit::WasmJitCpuState;
use crate::constants::{RAM_BASE, TIMER_CTL_ENABLE};
use crate::host::wasm::Emulator;
use crate::runtime::Machine;

#[test]
fn jit_scheduler_selects_next_runnable_core_and_exposes_its_pc() {
    let mut emulator = Emulator::new(Some(3));
    emulator.machine.cpus[0].lifecycle = CpuLifecycle::WaitingForInterrupt;
    emulator.machine.cpus[2].lifecycle = CpuLifecycle::Runnable;
    emulator.machine.cpus[2].regs.pc = RAM_BASE + 0x2000;
    emulator.machine.virtual_time = 17;

    assert_eq!(emulator.jit_prepare_next_core(), 2);
    assert_eq!(emulator.machine.active_core, 2);
    assert_eq!(emulator.machine.cpus[2].sys.cycle_count, 17);
    assert_eq!(emulator.pc_for_core(2), RAM_BASE + 0x2000);
    assert_eq!(emulator.pc_for_core(99), 0);
}

#[test]
fn jit_scheduler_fast_forwards_idle_time_before_selecting_a_woken_core() {
    let mut emulator = Emulator::new(Some(2));
    let cpu = &mut emulator.machine.cpus[0];
    cpu.lifecycle = CpuLifecycle::WaitingForInterrupt;
    cpu.sys.vbar_el1 = RAM_BASE + 0x8000;
    cpu.sys.cntv_ctl_el0 = TIMER_CTL_ENABLE;
    cpu.sys.cntv_cval_el0 = 23;

    assert_eq!(emulator.jit_prepare_next_core(), 0);
    assert_eq!(emulator.machine.virtual_time, 23);
    assert_eq!(emulator.machine.cpus[0].sys.cycle_count, 23);
    assert_eq!(emulator.machine.cpus[0].lifecycle, CpuLifecycle::Runnable);
}

#[test]
fn jit_scheduler_reports_no_runnable_core_with_minus_one() {
    let mut emulator = Emulator::new(Some(2));
    emulator.machine.cpus[0].lifecycle = CpuLifecycle::PoweredOff;

    assert_eq!(emulator.jit_prepare_next_core(), -1);
}

#[test]
fn non_prepared_multicore_commit_advances_exact_time_and_rotates() {
    let mut machine = Machine::new(3);
    machine.cpus[1].lifecycle = CpuLifecycle::Runnable;
    machine.cpus[2].lifecycle = CpuLifecycle::Runnable;
    machine.virtual_time = 10;
    assert_eq!(machine.prepare_next_core(), Some(0));
    let mut state = WasmJitCpuState::from_cpu(&machine.cpus[0]);
    state.pc = RAM_BASE + 16;

    commit_jit_state(&state, &mut machine, 0, 4, RAM_BASE + 16)
        .expect("multicore JIT commit should succeed");

    assert_eq!(machine.total_steps, 4);
    assert_eq!(machine.virtual_time, 14);
    assert_eq!(machine.cpus[0].sys.cycle_count, 14);
    assert_eq!(machine.active_core, 1);
}

#[test]
fn multicore_commit_skips_powered_off_cores_when_rotating() {
    let mut machine = Machine::new(3);
    machine.cpus[2].lifecycle = CpuLifecycle::Runnable;
    let mut state = WasmJitCpuState::from_cpu(&machine.cpus[0]);
    state.pc = RAM_BASE + 4;

    commit_jit_state(&state, &mut machine, 0, 1, RAM_BASE + 4)
        .expect("multicore JIT commit should succeed");

    assert_eq!(machine.active_core, 2);
}

#[test]
fn multicore_commit_rejects_the_wrong_active_core_without_mutation() {
    let mut machine = Machine::new(2);
    machine.cpus[1].lifecycle = CpuLifecycle::Runnable;
    let mut state = WasmJitCpuState::from_cpu(&machine.cpus[1]);
    state.pc = RAM_BASE + 4;

    let err = commit_jit_state(&state, &mut machine, 1, 1, RAM_BASE + 4)
        .expect_err("inactive core must not commit");

    assert!(err.contains("active core is 0, requested 1"), "{err}");
    assert_eq!(machine.total_steps, 0);
    assert_eq!(machine.virtual_time, 0);
    assert_eq!(machine.cpus[1].regs.pc, 0);
}

#[test]
fn jit_irq_preflight_is_targeted_to_the_requested_core() {
    let mut machine = Machine::new(2);
    machine.cpus[1].lifecycle = CpuLifecycle::Runnable;
    for cpu in &mut machine.cpus {
        cpu.pstate = cpu.pstate.with_irq_masked(false);
    }
    machine.bus.gic.enable_interrupt_for_cpu(1, 7);
    machine.bus.gic.set_pending_for_cpu(1, 7);

    can_commit_jit_block_now(&mut machine, 0, 1)
        .expect("an IRQ targeted to core 1 must not block core 0");

    machine.active_core = 1;
    let err = can_commit_jit_block_now(&mut machine, 1, 1)
        .expect_err("the targeted core must observe its pending IRQ");
    assert!(err.contains("pending IRQ boundary"), "{err}");
}

#[test]
fn prepared_multicore_commit_uses_the_same_time_and_rotation_rules() {
    let mut machine = Machine::new(2);
    machine.cpus[1].lifecycle = CpuLifecycle::Runnable;
    machine.virtual_time = 30;
    assert_eq!(machine.prepare_next_core(), Some(0));
    let mut state = WasmJitCpuState::from_cpu(&machine.cpus[0]);
    state.pc = RAM_BASE + 12;

    commit_finished_jit_state(&state, &mut machine, 0, 3, RAM_BASE + 12, true)
        .expect("prepared multicore JIT commit should succeed");

    assert_eq!(machine.total_steps, 3);
    assert_eq!(machine.virtual_time, 33);
    assert_eq!(machine.cpus[0].sys.cycle_count, 33);
    assert_eq!(machine.active_core, 1);
}

#[test]
fn prepared_multicore_commit_rejects_the_wrong_active_core() {
    let mut machine = Machine::new(2);
    machine.cpus[1].lifecycle = CpuLifecycle::Runnable;
    let mut state = WasmJitCpuState::from_cpu(&machine.cpus[1]);
    state.pc = RAM_BASE + 4;

    let err = commit_finished_jit_state(&state, &mut machine, 1, 1, RAM_BASE + 4, true)
        .expect_err("prepared inactive core must not commit");

    assert!(err.contains("active core is 0, requested 1"), "{err}");
    assert_eq!(machine.total_steps, 0);
    assert_eq!(machine.cpus[1].regs.pc, 0);
}
