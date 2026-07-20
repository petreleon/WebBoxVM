use super::*;
use std::sync::atomic::Ordering;

pub(super) fn initial_deadline(cpu: &Armv8Cpu, lifecycle: u8) -> u64 {
    if matches!(lifecycle, LIFE_WAITING | LIFE_WAITING_EVENT) {
        waiting_deadline(cpu)
    } else {
        NO_DEADLINE
    }
}

pub(super) fn has_runnable_peer(core: usize, control: &WasmParallelControl) -> bool {
    control
        .lifecycle
        .iter()
        .enumerate()
        .any(|(candidate, state)| {
            candidate != core
                && matches!(
                    state.load(Ordering::Acquire),
                    LIFE_RUNNABLE | LIFE_STARTING | LIFE_BOOT_READY
                )
        })
}

pub(super) fn locked_poll_due(
    core: usize,
    cpu: &Armv8Cpu,
    lifecycle: u8,
    last_poll_cycle: u64,
    control: &WasmParallelControl,
) -> bool {
    let now = control.next_cycle.load(Ordering::Acquire);
    !has_runnable_peer(core, control)
        || cpu.sys.irq_pending
        || (lifecycle == LIFE_WAITING_EVENT
            && control.event_registers[core].load(Ordering::Acquire))
        || control.deadlines[core].load(Ordering::Acquire) <= now
        || now.wrapping_sub(last_poll_cycle) >= IDLE_LOCK_POLL_CYCLES
}

pub(super) fn wake_if_ready(
    core: usize,
    cpu: &mut Armv8Cpu,
    control: &WasmParallelControl,
) -> bool {
    let _guard = control
        .gate
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    if control.system_off.load(Ordering::Acquire) {
        return false;
    }
    cpu.sys.cycle_count = control.next_cycle.load(Ordering::Acquire);
    let state = control.lifecycle[core].load(Ordering::Acquire);
    let event_wake =
        state == LIFE_WAITING_EVENT && control.event_registers[core].swap(false, Ordering::AcqRel);
    if !event_wake && !has_wake_event_locked(core, cpu, control) {
        publish_deadline(core, cpu, control);
        return false;
    }
    cpu.event_register = control.event_registers[core].load(Ordering::Acquire);
    cpu.waiting_for_event = false;
    cpu.lifecycle = CpuLifecycle::Runnable;
    control.deadlines[core].store(NO_DEADLINE, Ordering::Release);
    control.lifecycle[core].store(LIFE_RUNNABLE, Ordering::Release);
    true
}

pub(super) fn coordinate(core: usize, cpu: &Armv8Cpu, control: &WasmParallelControl) -> bool {
    if control.stop.load(Ordering::Acquire)
        || control.system_off.load(Ordering::Acquire)
        || control.remaining.load(Ordering::Acquire) == 0
    {
        control.stop.store(true, Ordering::Release);
        return true;
    }
    let _guard = control
        .gate
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    // This gate linearizes park/wake, bus refresh, and virtual-time advancement.
    if control.lifecycle.iter().any(|state| {
        matches!(
            state.load(Ordering::Acquire),
            LIFE_RUNNABLE | LIFE_STARTING | LIFE_BOOT_READY
        )
    }) {
        std::hint::spin_loop();
        return false;
    }
    let bus = unsafe { &mut *bus_ptr(control) };
    bus.refresh_interrupts();
    if control.lifecycle.iter().enumerate().any(|(core, state)| {
        matches!(
            state.load(Ordering::Acquire),
            LIFE_WAITING | LIFE_WAITING_EVENT
        ) && bus.gic.has_pending_enabled_for_cpu(core)
    }) {
        std::hint::spin_loop();
        return false;
    }
    publish_deadline(core, cpu, control);
    let deadline = control
        .deadlines
        .iter()
        .map(|value| value.load(Ordering::Acquire))
        .min()
        .unwrap_or(NO_DEADLINE);
    if deadline == NO_DEADLINE {
        control.stop.store(true, Ordering::Release);
        true
    } else {
        control.next_cycle.fetch_max(deadline, Ordering::AcqRel);
        false
    }
}

pub(super) fn has_wake_event_locked(
    core: usize,
    cpu: &Armv8Cpu,
    control: &WasmParallelControl,
) -> bool {
    if cpu.sys.irq_pending || cpu.sys.timer_irq_check_needed() {
        return true;
    }
    let bus = unsafe { &mut *bus_ptr(control) };
    bus.refresh_interrupts();
    bus.gic.has_pending_enabled_for_cpu(core)
}

fn publish_deadline(core: usize, cpu: &Armv8Cpu, control: &WasmParallelControl) {
    let deadline = if matches!(
        control.lifecycle[core].load(Ordering::Acquire),
        LIFE_WAITING | LIFE_WAITING_EVENT
    ) {
        waiting_deadline(cpu)
    } else {
        NO_DEADLINE
    };
    control.deadlines[core].store(deadline, Ordering::Release);
}

fn waiting_deadline(cpu: &Armv8Cpu) -> u64 {
    cpu.sys.next_timer_deadline().unwrap_or(NO_DEADLINE)
}

unsafe fn bus_ptr(control: &WasmParallelControl) -> *mut SystemBus {
    control.bus_ptr.load(Ordering::Acquire) as usize as *mut SystemBus
}
