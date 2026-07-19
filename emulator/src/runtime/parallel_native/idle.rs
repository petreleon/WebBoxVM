use super::*;
use std::sync::atomic::Ordering;

pub(super) fn initial_deadline(cpu: &Armv8Cpu) -> u64 {
    if cpu.lifecycle == CpuLifecycle::WaitingForInterrupt {
        cpu.sys.next_timer_deadline().unwrap_or(NO_DEADLINE)
    } else {
        NO_DEADLINE
    }
}

pub(in crate::runtime) fn wake_if_ready(
    core: usize,
    cpu: &mut Armv8Cpu,
    shared: &SharedRun<'_>,
) -> bool {
    let guard = shared
        .idle_gate
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    if shared.system_off.load(Ordering::Acquire) {
        cpu.lifecycle = CpuLifecycle::PoweredOff;
        return false;
    }
    cpu.sys.cycle_count = shared.next_cycle.load(Ordering::Acquire);
    let state = shared.lifecycle[core].load(Ordering::Acquire);
    let event_wake =
        state == LIFE_WAITING_EVENT && shared.event_registers[core].swap(false, Ordering::AcqRel);
    if !event_wake && !has_wake_event(core, cpu, shared) {
        publish_deadline(core, cpu, shared);
        drop(guard);
        std::thread::yield_now();
        return false;
    }
    cpu.event_register = shared.event_registers[core].load(Ordering::Acquire);
    cpu.waiting_for_event = false;
    cpu.lifecycle = CpuLifecycle::Runnable;
    shared.deadlines[core].store(NO_DEADLINE, Ordering::Release);
    shared.lifecycle[core].store(LIFE_RUNNABLE, Ordering::Release);
    true
}

pub(super) fn has_wake_event(core: usize, cpu: &Armv8Cpu, shared: &SharedRun<'_>) -> bool {
    if cpu.sys.irq_pending || cpu.sys.timer_irq_check_needed() {
        return true;
    }
    let mut bus = shared
        .bus
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    bus.refresh_interrupts();
    bus.gic.has_pending_enabled_for_cpu(core)
}

pub(in crate::runtime) fn coordinate(core: usize, cpu: &Armv8Cpu, shared: &SharedRun<'_>) -> bool {
    if shared.remaining.load(Ordering::Acquire) == 0 {
        shared.stop.store(true, Ordering::Release);
        return true;
    }
    let guard = shared
        .idle_gate
        .read()
        .unwrap_or_else(|poison| poison.into_inner());
    if shared.lifecycle.iter().any(|state| {
        matches!(
            state.load(Ordering::Acquire),
            LIFE_RUNNABLE | LIFE_STARTING | LIFE_BOOT_READY
        )
    }) {
        drop(guard);
        std::thread::yield_now();
        return false;
    }
    if waiting_irq_pending(shared) {
        drop(guard);
        std::thread::yield_now();
        return false;
    }
    publish_deadline(core, cpu, shared);
    let deadline = shared
        .deadlines
        .iter()
        .map(|value| value.load(Ordering::Acquire))
        .min()
        .unwrap_or(NO_DEADLINE);
    if deadline == NO_DEADLINE {
        shared.stop.store(true, Ordering::Release);
        true
    } else {
        shared.next_cycle.fetch_max(deadline, Ordering::AcqRel);
        false
    }
}

fn waiting_irq_pending(shared: &SharedRun<'_>) -> bool {
    let mut bus = shared
        .bus
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    bus.refresh_interrupts();
    shared.lifecycle.iter().enumerate().any(|(core, state)| {
        matches!(
            state.load(Ordering::Acquire),
            LIFE_WAITING | LIFE_WAITING_EVENT
        ) && bus.gic.has_pending_enabled_for_cpu(core)
    })
}

fn publish_deadline(core: usize, cpu: &Armv8Cpu, shared: &SharedRun<'_>) {
    let deadline = if matches!(
        shared.lifecycle[core].load(Ordering::Acquire),
        LIFE_WAITING | LIFE_WAITING_EVENT
    ) {
        cpu.sys.next_timer_deadline().unwrap_or(NO_DEADLINE)
    } else {
        NO_DEADLINE
    };
    shared.deadlines[core].store(deadline, Ordering::Release);
}
