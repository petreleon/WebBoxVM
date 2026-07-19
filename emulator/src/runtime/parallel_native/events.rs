use super::*;
use std::sync::atomic::Ordering;

pub(in crate::runtime) fn park_after_wait(
    core: usize,
    cpu: &mut Armv8Cpu,
    shared: &SharedRun<'_>,
    waiting_for_event: bool,
) {
    let _guard = shared
        .idle_gate
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    if shared.system_off.load(Ordering::Acquire) {
        shared.deadlines[core].store(NO_DEADLINE, Ordering::Release);
        cpu.lifecycle = CpuLifecycle::PoweredOff;
        return;
    }
    if waiting_for_event {
        let registered = shared.event_registers[core].swap(false, Ordering::AcqRel);
        cpu.event_register = false;
        if registered {
            cpu.waiting_for_event = false;
            return;
        }
    }
    if idle::has_wake_event(core, cpu, shared) {
        cpu.waiting_for_event = false;
        return;
    }
    shared.deadlines[core].store(
        cpu.sys.next_timer_deadline().unwrap_or(NO_DEADLINE),
        Ordering::Release,
    );
    let state = if waiting_for_event {
        LIFE_WAITING_EVENT
    } else {
        LIFE_WAITING
    };
    if shared.lifecycle[core]
        .compare_exchange(LIFE_RUNNABLE, state, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        cpu.waiting_for_event = waiting_for_event;
        cpu.lifecycle = CpuLifecycle::WaitingForInterrupt;
    } else {
        shared.deadlines[core].store(NO_DEADLINE, Ordering::Release);
        cpu.lifecycle = CpuLifecycle::PoweredOff;
    }
}

pub(super) fn set_local(core: usize, cpu: &mut Armv8Cpu, shared: &SharedRun<'_>) {
    cpu.event_register = true;
    shared.event_registers[core].store(true, Ordering::Release);
}

pub(in crate::runtime) fn broadcast(core: usize, cpu: &mut Armv8Cpu, shared: &SharedRun<'_>) {
    let _guard = shared
        .idle_gate
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    signal_targets(None, shared);
    cpu.event_register = shared.event_registers[core].load(Ordering::Acquire);
}

pub(super) fn signal_remote_store(sender: usize, shared: &SharedRun<'_>) {
    let _guard = shared
        .idle_gate
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    // The parallel monitor model uses one address-agnostic memory epoch.
    // Mirror that conservative invalidation scope in the generated events.
    signal_targets(Some(sender), shared);
}

fn signal_targets(excluded: Option<usize>, shared: &SharedRun<'_>) {
    for (core, state) in shared.lifecycle.iter().enumerate() {
        if Some(core) == excluded {
            continue;
        }
        shared.event_registers[core].store(true, Ordering::Release);
        if state
            .compare_exchange(
                LIFE_WAITING_EVENT,
                LIFE_RUNNABLE,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
        {
            shared.event_registers[core].store(false, Ordering::Relaxed);
            shared.deadlines[core].store(NO_DEADLINE, Ordering::Release);
        }
    }
}
