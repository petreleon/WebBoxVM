use super::*;
use std::sync::atomic::Ordering;

pub(super) fn park_after_wait(
    core: usize,
    cpu: &mut Armv8Cpu,
    control: &WasmParallelControl,
    waiting_for_event: bool,
) {
    let _guard = control
        .gate
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    if control.system_off.load(Ordering::Acquire) {
        return;
    }
    if waiting_for_event {
        let registered = control.event_registers[core].swap(false, Ordering::AcqRel);
        cpu.event_register = false;
        if registered {
            cpu.waiting_for_event = false;
            return;
        }
    }
    if idle::has_wake_event_locked(core, cpu, control) {
        cpu.waiting_for_event = false;
        return;
    }
    cpu.waiting_for_event = waiting_for_event;
    cpu.lifecycle = CpuLifecycle::WaitingForInterrupt;
    control.deadlines[core].store(
        cpu.sys.next_timer_deadline().unwrap_or(NO_DEADLINE),
        Ordering::Relaxed,
    );
    control.lifecycle[core].store(
        if waiting_for_event {
            LIFE_WAITING_EVENT
        } else {
            LIFE_WAITING
        },
        Ordering::Release,
    );
}

pub(super) fn set_local(core: usize, cpu: &mut Armv8Cpu, control: &WasmParallelControl) {
    cpu.event_register = true;
    control.event_registers[core].store(true, Ordering::Release);
}

pub(super) fn broadcast(sender: usize, cpu: &mut Armv8Cpu, control: &WasmParallelControl) {
    let _guard = control
        .gate
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    signal_targets(None, control);
    cpu.event_register = control.event_registers[sender].load(Ordering::Acquire);
}

pub(super) fn signal_remote_store_locked(sender: usize, control: &WasmParallelControl) {
    // The parallel monitor model uses one address-agnostic memory epoch.
    // Mirror that conservative invalidation scope in the generated events.
    signal_targets(Some(sender), control);
}

fn signal_targets(excluded: Option<usize>, control: &WasmParallelControl) {
    for (core, state) in control.lifecycle.iter().enumerate() {
        if Some(core) == excluded {
            continue;
        }
        control.event_registers[core].store(true, Ordering::Release);
        if state
            .compare_exchange(
                LIFE_WAITING_EVENT,
                LIFE_RUNNABLE,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
        {
            control.event_registers[core].store(false, Ordering::Relaxed);
            control.deadlines[core].store(NO_DEADLINE, Ordering::Release);
        }
    }
}
