use super::*;
use std::sync::atomic::Ordering;

#[test]
fn waiting_worker_throttles_locked_polls_while_a_peer_runs() {
    let mut machine = Machine::new(2);
    machine.cpus[0].lifecycle = CpuLifecycle::WaitingForInterrupt;
    machine.cpus[0].sys.cycle_count = 100;
    machine.cpus[1].lifecycle = CpuLifecycle::Runnable;
    let token = begin_parallel(&mut machine, 100);
    let control = &machine.wasm_parallel;
    let lifecycle = control.lifecycle[0].load(Ordering::Acquire);
    let now = control.next_cycle.load(Ordering::Acquire);

    assert!(idle::has_runnable_peer(0, control));
    assert!(!idle::locked_poll_due(
        0,
        &machine.cpus[0],
        lifecycle,
        now,
        control,
    ));
    control
        .next_cycle
        .store(now + IDLE_LOCK_POLL_CYCLES, Ordering::Release);
    assert!(idle::locked_poll_due(
        0,
        &machine.cpus[0],
        lifecycle,
        now,
        control,
    ));

    Machine::cancel_parallel_wasm(token).unwrap();
    Machine::finish_parallel_wasm(token).unwrap();
}

#[test]
fn waiting_worker_poll_hints_bypass_peer_throttling() {
    let mut machine = Machine::new(2);
    machine.cpus[0].lifecycle = CpuLifecycle::WaitingForInterrupt;
    machine.cpus[0].waiting_for_event = true;
    machine.cpus[1].lifecycle = CpuLifecycle::Runnable;
    let token = begin_parallel(&mut machine, 100);
    let control = &machine.wasm_parallel;
    let lifecycle = control.lifecycle[0].load(Ordering::Acquire);
    let now = control.next_cycle.load(Ordering::Acquire);

    control.event_registers[0].store(true, Ordering::Release);
    assert!(idle::locked_poll_due(
        0,
        &machine.cpus[0],
        lifecycle,
        now,
        control,
    ));

    Machine::cancel_parallel_wasm(token).unwrap();
    Machine::finish_parallel_wasm(token).unwrap();
}
