use super::*;

const STR_X0_X1: u64 = 0xf900_0020;
const LDXR_X0_X2: u64 = 0xc85f_7c40;
const WFE: u64 = 0xd503_205f;
const NOP: u64 = 0xd503_201f;
const CODE: u64 = RAM_BASE + 0x78_000;
const DATA: u64 = RAM_BASE + 0x79_000;

fn store_with_waiters() -> Machine {
    let mut machine = Machine::new(3);
    machine.bus.mem.write(CODE, 4, STR_X0_X1).unwrap();
    machine.cpus[0].regs.pc = CODE;
    machine.cpus[0].regs.set_x(0, 0xfeed_face);
    machine.cpus[0].regs.set_x(1, DATA);
    machine.cpus[0].reserve_exclusive(DATA, 8);

    machine.cpus[1].reserve_exclusive(DATA, 8);
    machine.cpus[1].lifecycle = CpuLifecycle::WaitingForInterrupt;
    machine.cpus[1].waiting_for_event = true;

    machine.cpus[2].reserve_exclusive(DATA, 8);
    machine.cpus[2].lifecycle = CpuLifecycle::WaitingForInterrupt;
    machine
}

fn assert_store_event_result(machine: &Machine, exact_clear: bool) {
    assert!(!machine.cpus[0].event_register);
    assert_eq!(machine.cpus[1].lifecycle, CpuLifecycle::Runnable);
    assert!(!machine.cpus[1].waiting_for_event);
    assert!(!machine.cpus[1].event_register);

    assert_eq!(machine.cpus[2].lifecycle, CpuLifecycle::WaitingForInterrupt);
    assert!(!machine.cpus[2].waiting_for_event);
    assert!(machine.cpus[2].event_register);

    if exact_clear {
        assert!(machine.cpus[0].exclusive.is_none());
        assert!(machine.cpus[1].exclusive.is_none());
        assert!(machine.cpus[2].exclusive.is_none());
    }
}

#[test]
fn cooperative_store_to_remote_monitors_wakes_wfe_but_not_wfi() {
    let mut machine = store_with_waiters();

    assert_eq!(machine.run(1), 1);

    assert_store_event_result(&machine, true);
}

#[test]
fn monitored_lock_release_wakes_wfe_without_timer_event_stream() {
    let mut machine = Machine::new(2);
    let waiter_code = CODE + 0x100;
    machine.bus.mem.write(waiter_code, 4, LDXR_X0_X2).unwrap();
    machine.bus.mem.write(waiter_code + 4, 4, WFE).unwrap();
    machine.bus.mem.write(waiter_code + 8, 4, NOP).unwrap();
    machine.cpus[0].lifecycle = CpuLifecycle::PoweredOff;
    machine.cpus[1].lifecycle = CpuLifecycle::Runnable;
    machine.cpus[1].regs.pc = waiter_code;
    machine.cpus[1].regs.set_x(2, DATA);
    machine.cpus[1].sys.cntkctl_el1 = 0;
    machine.active_core = 1;

    assert_eq!(machine.run(1), 1);
    assert!(machine.cpus[1].exclusive_matches(DATA, 8));
    assert!(!machine.cpus[1].event_register);
    assert_eq!(machine.cpus[1].regs.pc, waiter_code + 4);
    assert!(!machine.core_has_wake_event(1));
    assert_eq!(machine.run(1), 1);
    assert_eq!(machine.cpus[1].lifecycle, CpuLifecycle::WaitingForInterrupt);
    assert!(machine.cpus[1].waiting_for_event);
    assert_eq!(machine.run(1), 0, "no timer event should wake the waiter");

    machine.bus.mem.write(CODE, 4, STR_X0_X1).unwrap();
    machine.cpus[0].lifecycle = CpuLifecycle::Runnable;
    machine.cpus[0].regs.pc = CODE;
    machine.cpus[0].regs.set_x(0, 1);
    machine.cpus[0].regs.set_x(1, DATA);
    machine.active_core = 0;

    assert_eq!(machine.run(1), 1);
    assert!(machine.cpus[1].exclusive.is_none());
    assert_eq!(machine.cpus[1].lifecycle, CpuLifecycle::Runnable);
    assert!(!machine.cpus[1].waiting_for_event);
    assert_eq!(
        machine.run(1),
        1,
        "the released waiter should make progress"
    );
    assert_eq!(machine.cpus[1].regs.pc, waiter_code + 12);
}

#[test]
fn cooperative_nonoverlapping_store_does_not_generate_an_event() {
    let mut machine = store_with_waiters();
    machine.cpus[0].regs.set_x(1, DATA + 0x100);

    assert_eq!(machine.run(1), 1);

    assert_eq!(machine.cpus[1].lifecycle, CpuLifecycle::WaitingForInterrupt);
    assert!(machine.cpus[1].waiting_for_event);
    assert!(!machine.cpus[1].event_register);
    assert!(machine.cpus[1].exclusive_matches(DATA, 8));
}

#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
#[test]
fn native_parallel_store_conservatively_wakes_wfe_but_not_wfi() {
    let mut machine = store_with_waiters();
    machine.set_run_backend(RunBackend::NativeThreads);

    assert_eq!(machine.run(1), 1);

    assert_store_event_result(&machine, false);
}
